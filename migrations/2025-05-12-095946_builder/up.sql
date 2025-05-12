-- Your SQL goes here

CREATE TABLE ejbuilder (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	ejclient_id uuid NOT NULL REFERENCES ejclient(id),
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('ejbuilder');
