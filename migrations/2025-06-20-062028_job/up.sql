-- Your SQL goes here

CREATE TABLE ejjobtype (
	id SERIAL PRIMARY KEY,
	job_type VARCHAR NOT NULL
);

INSERT INTO ejjobtype (id, job_type) VALUES 
	(0, 'Build'),
	(1, 'Run');

CREATE TABLE ejjobstatus (
	id SERIAL PRIMARY KEY,
	status VARCHAR NOT NULL
);

INSERT INTO ejjobstatus (id, status) VALUES 
	(0, 'Not started'),
	(1, 'Running'),
	(2, 'Success'),
	(3, 'Failed');

CREATE TABLE ejjob (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	commit_hash VARCHAR NOT NULL,
	remote_url VARCHAR NOT NULL,
	job_type INTEGER REFERENCES ejjobtype(id) NOT NULL,
	status INTEGER REFERENCES ejjobstatus(id) NOT NULL DEFAULT 0,
	dispatched_at TIMESTAMPTZ,
	finished_at TIMESTAMPTZ,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION update_ejjob_timestamps()
RETURNS TRIGGER AS $$
BEGIN
    -- Always update the updated_at timestamp
    NEW.updated_at = CURRENT_TIMESTAMP;
    
    -- If status is changing to 'Running' (1), set dispatched_at
    IF NEW.status = 1 AND (OLD.status IS NULL OR OLD.status != 1) THEN
        NEW.dispatched_at = CURRENT_TIMESTAMP;
    END IF;
    
    -- If status is changing to 'Success' (2) or 'Failed' (3), set finished_at
    IF NEW.status IN (2, 3) AND (OLD.status IS NULL OR OLD.status NOT IN (2, 3)) THEN
        NEW.finished_at = CURRENT_TIMESTAMP;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create the trigger
CREATE TRIGGER ejjob_status_timestamp_trigger
    BEFORE UPDATE ON ejjob
    FOR EACH ROW
    EXECUTE FUNCTION update_ejjob_timestamps();

SELECT diesel_manage_updated_at('ejjob');

CREATE TABLE ejjoblog (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	ejjob_id uuid REFERENCES ejjob(id) ON DELETE CASCADE NOT NULL,
	ejboard_config_id uuid REFERENCES ejboard_config(id) ON DELETE CASCADE NOT NULL,
	log VARCHAR NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('ejjoblog');

CREATE TABLE ejjobresult (
	ejjob_id uuid REFERENCES ejjob(id) ON DELETE CASCADE NOT NULL,
	ejboard_config_id uuid REFERENCES ejboard_config(id) ON DELETE CASCADE NOT NULL,
	result VARCHAR NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY (ejjob_id, ejboard_config_id)
);

SELECT diesel_manage_updated_at('ejjobresult');
