-- 0002 — trigram search index (external-content FTS5), synced to manga.title by triggers.
-- Trigram tokenizer makes CJK and substring search work: 'piece' matches 'One Piece'.

CREATE VIRTUAL TABLE manga_fts USING fts5(
    title,
    content = 'manga',
    content_rowid = 'id',
    tokenize = 'trigram'
);

CREATE TRIGGER manga_fts_insert AFTER INSERT ON manga BEGIN
    INSERT INTO manga_fts (rowid, title) VALUES (new.id, new.title);
END;

CREATE TRIGGER manga_fts_delete AFTER DELETE ON manga BEGIN
    INSERT INTO manga_fts (manga_fts, rowid, title) VALUES ('delete', old.id, old.title);
END;

CREATE TRIGGER manga_fts_update AFTER UPDATE OF title ON manga BEGIN
    INSERT INTO manga_fts (manga_fts, rowid, title) VALUES ('delete', old.id, old.title);
    INSERT INTO manga_fts (rowid, title) VALUES (new.id, new.title);
END;
