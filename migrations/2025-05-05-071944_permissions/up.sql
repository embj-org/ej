-- Your SQL goes here

CREATE TABLE permission (
	id VARCHAR PRIMARY KEY
);

CREATE TABLE client_permission (
	ejclient_id uuid REFERENCES ejclient(id) ON DELETE CASCADE NOT NULL,
	permission_id VARCHAR REFERENCES permission(id) ON DELETE CASCADE NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY (ejclient_id, permission_id)
);

SELECT diesel_manage_updated_at('client_permission');
