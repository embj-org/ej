-- Your SQL goes here

CREATE TABLE ejconfig (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	ejbuilder_id uuid NOT NULL REFERENCES ejbuilder(id),
	version VARCHAR(50) NOT NULL,
	hash VARCHAR(255) NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE ejboard (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	ejconfig_id uuid NOT NULL REFERENCES ejconfig(id),
	name VARCHAR(255) NOT NULL,
	description TEXT NOT NULL DEFAULT '',
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE ejboard_config (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	ejboard_id uuid NOT NULL REFERENCES ejboard(id),
	name TEXT NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE ejtag (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	name VARCHAR(100) NOT NULL UNIQUE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE ejboard_config_tag (
	ejboard_config_id uuid NOT NULL REFERENCES ejboard_config(id),
	ejtag_id uuid NOT NULL REFERENCES ejtag(id),
	PRIMARY KEY (ejboard_config_id, ejtag_id)
);


SELECT diesel_manage_updated_at('ejconfig');
SELECT diesel_manage_updated_at('ejboard');
SELECT diesel_manage_updated_at('ejboard_config');
SELECT diesel_manage_updated_at('ejtag');
SELECT diesel_manage_updated_at('ejboard_config_tag');
