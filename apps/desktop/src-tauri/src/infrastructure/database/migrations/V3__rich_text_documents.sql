-- Card faces store versioned TipTap JSON documents in front_content / back_content.
-- Schema columns remain TEXT; legacy plain-text rows are normalized on read
-- (RichTextDocument::parse_stored) and rewritten on the next update.
--
-- Document shape: { "version": 1, "type": "doc", "content": [...] }
--
-- No destructive ALTER — this migration documents the content contract.
SELECT 1;
