ALTER TABLE files ADD version TEXT NOT NULL;
ALTER TABLE files DROP uploaded_on;
