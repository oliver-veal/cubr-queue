CREATE SCHEMA queue;

CREATE TABLE IF NOT EXISTS queue.queue (
    user_id        uuid       NOT NULL,
    id             text       NOT NULL,

    file_id        uuid       NOT NULL,
    file_version   integer    NOT NULL,

    frame_start    integer    NOT NULL,
    frame_end      integer    NOT NULL,
    step           integer    NOT NULL,
    slices         integer    NOT NULL,

    pointer_frame  integer    NOT NULL,
    pointer_slice  integer    NOT NULL,

    total_jobs     integer    NOT NULL,
    completed_jobs integer    NOT NULL,

    subscription_item_id text NOT NULL,

    PRIMARY KEY (user_id, id)
);

ALTER TABLE queue.queue ENABLE ROW LEVEL SECURITY;

CREATE TABLE IF NOT EXISTS queue.jobs (
    render_id text    NOT NULL,
    frame     integer NOT NULL,
    slice     integer NOT NULL,

    user_id   uuid    NOT NULL,
    worker_id text    NOT NULL,
    
    PRIMARY KEY (render_id, frame, slice)
);

ALTER TABLE queue.jobs ENABLE ROW LEVEL SECURITY;