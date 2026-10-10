pub const SEARCH_UPSERT: &str = "INSERT INTO search_index (
    id, profile_id, entity_type, entity_id, title, content, normalized_content,
    metadata_json, language, created_at, updated_at
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
ON CONFLICT(entity_type, entity_id) DO UPDATE SET
    title = excluded.title,
    content = excluded.content,
    normalized_content = excluded.normalized_content,
    metadata_json = excluded.metadata_json,
    language = excluded.language,
    updated_at = excluded.updated_at,
    profile_id = excluded.profile_id";

pub const SEARCH_REMOVE: &str =
    "DELETE FROM search_index WHERE entity_type = ?1 AND entity_id = ?2";

pub const SEARCH_REMOVE_TYPES_PREFIX: &str = "DELETE FROM search_index WHERE entity_type IN (";

pub const SEARCH_CLEAR: &str = "DELETE FROM search_index";

pub const SEARCH_COUNT: &str = "SELECT COUNT(*) FROM search_index";

pub const SEARCH_FTS: &str = "SELECT s.id, s.profile_id, s.entity_type, s.entity_id, s.title,
    s.content, s.normalized_content, s.metadata_json, s.language, s.created_at, s.updated_at
 FROM search_index_fts
 JOIN search_index s ON s.rowid = search_index_fts.rowid
 WHERE search_index_fts MATCH ?1
 ORDER BY bm25(search_index_fts)
 LIMIT ?2";

pub const SEARCH_FTS_TYPED: &str = "SELECT s.id, s.profile_id, s.entity_type, s.entity_id, s.title,
    s.content, s.normalized_content, s.metadata_json, s.language, s.created_at, s.updated_at
 FROM search_index_fts
 JOIN search_index s ON s.rowid = search_index_fts.rowid
 WHERE search_index_fts MATCH ?1
   AND s.entity_type IN (SELECT value FROM json_each(?2))
 ORDER BY bm25(search_index_fts)
 LIMIT ?3";

pub const SEARCH_FALLBACK: &str = "SELECT id, profile_id, entity_type, entity_id, title,
    content, normalized_content, metadata_json, language, created_at, updated_at
 FROM search_index
 WHERE normalized_content LIKE ?1 OR title LIKE ?2
 LIMIT ?3";

pub const SEARCH_FALLBACK_TYPED: &str = "SELECT id, profile_id, entity_type, entity_id, title,
    content, normalized_content, metadata_json, language, created_at, updated_at
 FROM search_index
 WHERE (normalized_content LIKE ?1 OR title LIKE ?2)
   AND entity_type IN (SELECT value FROM json_each(?3))
 LIMIT ?4";

pub const RECENT_LIST: &str = "SELECT id, profile_id, query, filters, created_at
 FROM recent_searches
 WHERE profile_id = ?1
 ORDER BY created_at DESC
 LIMIT ?2";

pub const RECENT_INSERT: &str =
    "INSERT INTO recent_searches (id, profile_id, query, filters, created_at)
 VALUES (?1, ?2, ?3, ?4, ?5)";

pub const RECENT_DELETE_QUERY: &str =
    "DELETE FROM recent_searches WHERE profile_id = ?1 AND query = ?2";

pub const RECENT_DELETE_ID: &str = "DELETE FROM recent_searches WHERE profile_id = ?1 AND id = ?2";

pub const RECENT_CLEAR: &str = "DELETE FROM recent_searches WHERE profile_id = ?1";

pub const RECENT_TRIM: &str = "DELETE FROM recent_searches
 WHERE profile_id = ?1
   AND id NOT IN (
     SELECT id FROM recent_searches
     WHERE profile_id = ?1
     ORDER BY created_at DESC
     LIMIT ?2
   )";

pub const HISTORY_INSERT: &str = "INSERT INTO search_history
 (id, profile_id, query, entity_type, entity_id, created_at)
 VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

pub const HISTORY_LIST: &str = "SELECT id, profile_id, query, entity_type, entity_id, created_at
 FROM search_history
 WHERE profile_id = ?1
 ORDER BY created_at DESC
 LIMIT ?2";

pub const HISTORY_CLEAR: &str = "DELETE FROM search_history WHERE profile_id = ?1";

pub const HISTORY_POPULAR: &str =
    "SELECT query, COUNT(*) AS hit_count, MAX(created_at) AS last_searched_at
 FROM search_history
 WHERE profile_id = ?1 AND TRIM(query) != ''
 GROUP BY query
 ORDER BY hit_count DESC, last_searched_at DESC
 LIMIT ?2";

pub const HISTORY_TRIM: &str = "DELETE FROM search_history
 WHERE profile_id = ?1
   AND id NOT IN (
     SELECT id FROM search_history
     WHERE profile_id = ?1
     ORDER BY created_at DESC
     LIMIT ?2
   )";
