use crate::error::CoreResult;
use crate::member::{MemberUpdate, NewMember, TeamMember};
use crate::repository::MemberRepository;
use crate::service::now;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct MemberService {
    repo: Arc<dyn MemberRepository>,
}

impl MemberService {
    pub fn new(repo: Arc<dyn MemberRepository>) -> Self {
        Self { repo }
    }

    pub async fn list(&self, project_id: Uuid) -> CoreResult<Vec<TeamMember>> {
        self.repo.list_members(project_id).await
    }

    pub async fn create(&self, input: NewMember) -> CoreResult<TeamMember> {
        let member = TeamMember::create(input, now())?;
        self.repo.insert_member(&member).await?;
        Ok(member)
    }

    pub async fn update(&self, input: MemberUpdate) -> CoreResult<TeamMember> {
        let mut member = self.repo.get_member(input.id).await?;
        member.apply(input, now())?;
        self.repo.update_member(&member).await?;
        Ok(member)
    }

    pub async fn delete(&self, id: Uuid) -> CoreResult<()> {
        self.repo.delete_member(id).await
    }
}
