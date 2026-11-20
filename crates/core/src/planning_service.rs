use crate::error::{CoreError, CoreResult};
use crate::planning::{
    DayItem, DayUpdate, ItemUpdate, NewBreakItem, NewDay, NewSceneItem, ShootingDay,
};
use crate::repository::{MemberRepository, PlanningRepository, SceneRepository};
use crate::schedule::{compute, Schedule};
use crate::service::now;
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// Tout ce dont l'écran d'une journée a besoin, calculé en une fois.
/// `items[i]` correspond à `schedule.slots[i]`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayView {
    pub day: ShootingDay,
    pub items: Vec<DayItem>,
    pub schedule: Schedule,
    /// Membres convoqués mais indisponibles à cette date.
    pub unavailable_member_ids: Vec<Uuid>,
}

#[derive(Clone)]
pub struct PlanningService {
    days: Arc<dyn PlanningRepository>,
    members: Arc<dyn MemberRepository>,
    scenes: Arc<dyn SceneRepository>,
}

fn next_position(items: &[DayItem]) -> u32 {
    items.iter().map(|i| i.position).max().map_or(0, |p| p + 1)
}

impl PlanningService {
    pub fn new(
        days: Arc<dyn PlanningRepository>,
        members: Arc<dyn MemberRepository>,
        scenes: Arc<dyn SceneRepository>,
    ) -> Self {
        Self { days, members, scenes }
    }

    pub async fn list_days(&self, project_id: Uuid) -> CoreResult<Vec<ShootingDay>> {
        self.days.list_days(project_id).await
    }

    async fn check_members(&self, project_id: Uuid, ids: &[Uuid]) -> CoreResult<()> {
        let known: HashSet<Uuid> = self
            .members
            .list_members(project_id)
            .await?
            .into_iter()
            .map(|m| m.id)
            .collect();
        if ids.iter().any(|id| !known.contains(id)) {
            return Err(CoreError::Validation(
                "Un des membres convoqués n'appartient pas à ce projet.".into(),
            ));
        }
        Ok(())
    }

    async fn check_date_free(
        &self,
        project_id: Uuid,
        date: NaiveDate,
        except: Option<Uuid>,
    ) -> CoreResult<()> {
        let taken = self
            .days
            .list_days(project_id)
            .await?
            .iter()
            .any(|d| d.date == date && Some(d.id) != except);
        if taken {
            return Err(CoreError::Validation(
                "Une journée de tournage existe déjà à cette date.".into(),
            ));
        }
        Ok(())
    }

    pub async fn create_day(&self, input: NewDay) -> CoreResult<ShootingDay> {
        let day = ShootingDay::create(input, now())?;
        self.check_members(day.project_id, &day.member_ids).await?;
        self.check_date_free(day.project_id, day.date, None).await?;
        self.days.insert_day(&day).await?;
        Ok(day)
    }

    pub async fn update_day(&self, input: DayUpdate) -> CoreResult<ShootingDay> {
        let mut day = self.days.get_day(input.id).await?;
        day.apply(input, now())?;
        self.check_members(day.project_id, &day.member_ids).await?;
        self.check_date_free(day.project_id, day.date, Some(day.id)).await?;
        self.days.update_day(&day).await?;
        Ok(day)
    }

    pub async fn delete_day(&self, id: Uuid) -> CoreResult<()> {
        self.days.delete_day(id).await
    }

    pub async fn view(&self, day_id: Uuid) -> CoreResult<DayView> {
        let day = self.days.get_day(day_id).await?;
        let items = self.days.list_items(day_id).await?;
        let schedule = compute(u32::from(day.start_min), u32::from(day.end_min), &items);
        let unavailable_member_ids = self
            .members
            .list_members(day.project_id)
            .await?
            .into_iter()
            .filter(|m| day.member_ids.contains(&m.id) && !m.is_available_on(day.date))
            .map(|m| m.id)
            .collect();
        Ok(DayView { day, items, schedule, unavailable_member_ids })
    }

    pub async fn add_scene(&self, input: NewSceneItem) -> CoreResult<DayItem> {
        let day = self.days.get_day(input.day_id).await?;
        let scene = self.scenes.get(input.scene_id).await?;
        if scene.project_id != day.project_id {
            return Err(CoreError::Validation(
                "Cette scène n'appartient pas à ce projet.".into(),
            ));
        }
        let items = self.days.list_items(day.id).await?;
        if items.iter().any(|i| i.scene_id == Some(scene.id)) {
            return Err(CoreError::Validation(
                "Cette scène est déjà dans la journée.".into(),
            ));
        }
        let duration = input.duration_min.or(scene.estimated_duration_min).unwrap_or(30);
        let item = DayItem::new_scene(day.id, scene.id, duration, next_position(&items))?;
        self.days.insert_item(&item).await?;
        Ok(item)
    }

    pub async fn add_break(&self, input: NewBreakItem) -> CoreResult<DayItem> {
        let items = self.days.list_items(input.day_id).await?;
        self.days.get_day(input.day_id).await?;
        let item = DayItem::new_break(input, next_position(&items))?;
        self.days.insert_item(&item).await?;
        Ok(item)
    }

    pub async fn update_item(&self, input: ItemUpdate) -> CoreResult<DayItem> {
        let mut item = self.days.get_item(input.id).await?;
        item.apply(input)?;
        self.days.update_item(&item).await?;
        Ok(item)
    }

    pub async fn reorder(&self, day_id: Uuid, ordered: Vec<Uuid>) -> CoreResult<()> {
        let mut current: Vec<Uuid> = self
            .days
            .list_items(day_id)
            .await?
            .iter()
            .map(|i| i.id)
            .collect();
        let mut wanted = ordered.clone();
        current.sort();
        wanted.sort();
        if current != wanted {
            return Err(CoreError::Validation("Ordre des éléments invalide.".into()));
        }
        self.days.set_item_order(day_id, &ordered).await
    }

    pub async fn remove_item(&self, id: Uuid) -> CoreResult<()> {
        self.days.delete_item(id).await
    }
}
