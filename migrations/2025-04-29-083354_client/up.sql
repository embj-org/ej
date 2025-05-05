-- Your SQL goes here

CREATE TABLE ejclient (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	name varchar NOT NULL UNIQUE,
	hash varchar(255) NOT NULL,
	hash_version integer NOT NULL DEFAULT 1,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('ejclient');
