create table limits (
    user_id uuid PRIMARY KEY NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    max_documents integer NOT NULL DEFAULT 100,
	max_size bigint NOT NULL DEFAULT 1048576000,
	current_documents integer NOT NULL DEFAULT 0,
	current_size bigint NOT NULL DEFAULT 0
);