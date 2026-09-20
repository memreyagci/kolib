use std::str::FromStr;

use crate::{
    archive::model::Archive,
    error::ContactError,
    export_reader::contacts::models::{Contact, ContactId},
};

impl Contact {
    pub async fn rename(&mut self, archive: &Archive, new_name: &str) -> Result<(), ContactError> {
        if self.is_me() {
            return Err(ContactError::CannotRenameMe);
        }

        Self::validate_name(new_name)?;

        let contact_id = self.id().to_string();
        let result = sqlx::query!(
            "UPDATE contacts SET name = ? WHERE id = ? AND is_me = 0",
            new_name,
            contact_id,
        )
        .execute(archive.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(ContactError::NotFound { contact_id });
        }

        self.set_name(new_name.to_owned());

        Ok(())
    }

    pub async fn delete(self, archive: &Archive) -> Result<(), ContactError> {
        if self.is_me() {
            return Err(ContactError::CannotDeleteMe);
        }

        let contact_id = self.id().to_string();
        let result = sqlx::query!(
            "DELETE FROM contacts WHERE id = ? AND is_me = 0",
            contact_id,
        )
        .execute(archive.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(ContactError::NotFound { contact_id });
        }

        Ok(())
    }

    pub async fn get_by_id(archive: &Archive, id: &ContactId) -> Result<Self, ContactError> {
        let contact_id = id.to_string();
        let row = sqlx::query!(
            "SELECT id, name, is_me FROM contacts WHERE id = ?",
            contact_id,
        )
        .fetch_optional(archive.pool())
        .await?
        .ok_or(ContactError::NotFound { contact_id })?;

        Ok(Self::new(
            ContactId::from_str(&row.id)?,
            row.name,
            row.is_me != 0,
        ))
    }

    pub async fn get_all(archive: &Archive) -> Result<Vec<Self>, ContactError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, name, is_me
            FROM contacts
            ORDER BY is_me DESC, name COLLATE NOCASE, id
            "#,
        )
        .fetch_all(archive.pool())
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(Self::new(
                    ContactId::from_str(&row.id)?,
                    row.name,
                    row.is_me != 0,
                ))
            })
            .collect()
    }

    pub async fn get_me(archive: &Archive) -> Result<Self, ContactError> {
        let row = sqlx::query!("SELECT id, name, is_me FROM contacts WHERE is_me = 1")
            .fetch_optional(archive.pool())
            .await?
            .ok_or(ContactError::MeNotFound)?;

        Ok(Self::new(
            ContactId::from_str(&row.id)?,
            row.name,
            row.is_me != 0,
        ))
    }
}
