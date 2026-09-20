use crate::{
    archive::model::Archive,
    error::ContactError,
    export_reader::contacts::models::{Contact, ContactId},
};

impl Contact {
    pub async fn create(archive: &Archive, name: &str) -> Result<Self, ContactError> {
        Self::validate_name(name)?;

        let contact = Self::new(ContactId::new(), name.to_owned(), false);
        let contact_id = contact.id().to_string();

        sqlx::query!(
            "INSERT INTO contacts (id, name, is_me) VALUES (?, ?, 0)",
            contact_id,
            name,
        )
        .execute(archive.pool())
        .await?;

        Ok(contact)
    }
}
