# Twitter direct-messages.js fixtures

- `comprehensive/` is a valid import covering conversations, reactions, edits, external URLs, supported local-media URL shapes, copied media, intentionally missing media, and a message combining every supported child-data type.
- `empty/` is a valid export containing no conversations. This must not give an invalid file error.
- `invalid_json/` has malformed JSON and must fail during decoding.
- `missing_required_fields/` is valid JSON but omits the required `senderId` field and must fail during schema deserialization.
- `missing_optional_fields/` is valid JSON that omits the optional `editHistory` field and must import successfully with no edit-history rows.
- `duplicate_ids/` is structurally valid but repeats a platform message ID and must fail without partially committing the import.

A missing media referenced by the comprehensive fixture is intentionally not present in its `direct_messages_media/` directory.
