DROP TABLE IF EXISTS gallery CASCADE;
DROP TABLE IF EXISTS gallery_rag_embeddings CASCADE;

CREATE TABLE IF NOT EXISTS gallery_rag_embeddings(
    id bigserial primary key NOT null,
    path text,
    keywords text[],
    description text, 
    theme text,
    img_aria text,
    img_alt text,
    embedding vector(512) NOT null,
    created_at timestamptz NOT null default now()
);

CREATE TABLE IF NOT EXISTS gallery(
    id uuid primary key default gen_random_uuid(),
    path text not null, 
    thumbnail_path text, 
    thumbnail_height int,
    thumbnail_width int,
    thumbnail_ratio text,
    embeddings_id bigint REFERENCES gallery_rag_embeddings(id) ON DELETE CASCADE, 
    created_at timestamptz NOT null default now(),
    updated_at timestamptz NOT null default now()
);

CREATE INDEX IF NOT EXISTS gallery_rag_embeddings_idx 
    ON  gallery_rag_embeddings  
    USING diskann (embedding);

CREATE TABLE IF NOT EXISTS user_upload(
    id uuid primary key default gen_random_uuid(),
    filename text NOT null,
    filesize int NOT null,
    filehash text NOT null,
    original_filename text NOT null,
    user_id text,
    gallery_id uuid,
    created_at timestamptz NOT null default now(),
    updated_at timestamptz NOT null default now()
);
