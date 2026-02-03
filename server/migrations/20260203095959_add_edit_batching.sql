-- Add pending_edit_batches table for batching coordinate edit submissions
CREATE TABLE pending_edit_batches (
    id SERIAL PRIMARY KEY,
    edit_data JSONB NOT NULL,
    token_id VARCHAR(255) NOT NULL,
    submitted_at TIMESTAMP NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMP,
    batch_pr_url TEXT,
    status VARCHAR(50) DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed'))
);

CREATE INDEX idx_pending_status ON pending_edit_batches(status, submitted_at);
CREATE INDEX idx_submitted_at ON pending_edit_batches(submitted_at) WHERE status = 'pending';
