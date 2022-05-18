CREATE TABLE maps (
    map_id         INT4 NOT NULL,
    mapset_id      INT4 NOT NULL,
    user_id        INT4 NOT NULL DEFAULT 0,
    checksum       VARCHAR(32),
    version        VARCHAR(80) NOT NULL DEFAULT '',
    seconds_total  INT4 NOT NULL DEFAULT 0,
    seconds_drain  INT4 NOT NULL DEFAULT 0,
    count_circles  INT4 NOT NULL DEFAULT 0,
    count_sliders  INT4 NOT NULL DEFAULT 0,
    count_spinners INT4 NOT NULL DEFAULT 0,
    hp             FLOAT4 NOT NULL DEFAULT 0,
    cs             FLOAT4 NOT NULL DEFAULT 0,
    od             FLOAT4 NOT NULL DEFAULT 0,
    ar             FLOAT4 NOT NULL DEFAULT 0,
    mode           INT2 NOT NULL DEFAULT 0,
    status         INT2 NOT NULL DEFAULT 0,
    last_update    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stars          FLOAT4 NOT NULL DEFAULT 0,
    bpm            FLOAT4 NOT NULL DEFAULT 0,
    max_combo      INT4,

    PRIMARY KEY (map_id)
);

CREATE INDEX maps_mapset_id ON maps (mapset_id);
CREATE INDEX maps_user_id ON maps (user_id);

CREATE TABLE mapsets (
    mapset_id   INT4 NOT NULL,
    user_id     INT4 NOT NULL DEFAULT 0,
    artist      VARCHAR(80) NOT NULL DEFAULT '',
    title       VARCHAR(80) NOT NULL DEFAULT '',
    creator     VARCHAR(80) NOT NULL DEFAULT '',
    bpm         FLOAT4 NOT NULL DEFAULT 0,
    status      INT2 NOT NULL DEFAULT 0,
    ranked_date TIMESTAMPTZ NOT NULL,

    PRIMARY KEY (mapset_id)
);

CREATE INDEX mapsets_user_id ON mapsets (user_id);
CREATE INDEX mapsets_status ON mapsets (status);

CREATE TABLE guild_configs (
    guild_id INT8 NOT NULL,
    prefixes      JSON NOT NULL,

    PRIMARY KEY (guild_id)
);