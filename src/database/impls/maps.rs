use std::{error::Error as StdError, fmt};

use futures::future::{BoxFuture, FutureExt};
use rosu_v2::prelude::{
    Beatmap, Beatmapset, GameMode,
    RankStatus::{Approved, Loved, Ranked},
};
use sqlx::{Error as SqlxError, PgConnection};
use thiserror::Error;

use crate::{
    database::{DBBeatmap, DBBeatmapset, Database},
    BotResult,
};

macro_rules! invalid_status {
    ($obj:ident) => {
        !matches!($obj.status, Ranked | Loved | Approved)
    };
}

type InsertMapResult<T> = Result<T, InsertMapOrMapsetError>;

#[derive(Debug)]
pub enum InsertMapOrMapsetError {
    Map(InsertMapError),
    Mapset(InsertMapsetError),
    Sqlx(SqlxError),
}

impl From<InsertMapError> for InsertMapOrMapsetError {
    fn from(err: InsertMapError) -> Self {
        Self::Map(err)
    }
}

impl From<InsertMapsetError> for InsertMapOrMapsetError {
    fn from(err: InsertMapsetError) -> Self {
        Self::Mapset(err)
    }
}

impl From<SqlxError> for InsertMapOrMapsetError {
    fn from(err: SqlxError) -> Self {
        Self::Sqlx(err)
    }
}

impl StdError for InsertMapOrMapsetError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Map(err) => err.source(),
            Self::Mapset(err) => err.source(),
            Self::Sqlx(err) => Some(err),
        }
    }
}

impl fmt::Display for InsertMapOrMapsetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Map(err) => write!(f, "{err}"),
            Self::Mapset(err) => write!(f, "{err}"),
            Self::Sqlx(_) => f.write_str("sqlx error"),
        }
    }
}

#[derive(Debug, Error)]
pub enum InsertMapError {
    #[error("cannot add {0:?} map to DB without combo")]
    MissingCombo(GameMode),
    #[error("failed to add map to DB")]
    Sqlx(#[from] SqlxError),
}

#[derive(Debug, Error)]
pub enum InsertMapsetError {
    #[error("failed to add mapset to DB")]
    Sqlx(#[from] SqlxError),
}

fn should_not_be_stored(map: &Beatmap) -> bool {
    invalid_status!(map) || map.convert || (map.mode != GameMode::Mania && map.max_combo.is_none())
}

impl Database {
    pub async fn get_beatmap(&self, map_id: u32, with_mapset: bool) -> BotResult<Beatmap> {
        let mut conn = self.pool.acquire().await?;

        let query = sqlx::query_as!(
            DBBeatmap,
            "SELECT * FROM maps WHERE map_id=$1",
            map_id as i32
        );

        let row = query.fetch_one(&mut conn).await?;
        let mut map = Beatmap::from(row);

        if with_mapset {
            let query = sqlx::query_as!(
                DBBeatmapset,
                "SELECT * FROM mapsets WHERE mapset_id=$1",
                map.mapset_id as i32
            );

            let mapset = query.fetch_one(&mut conn).await?;

            map.mapset.replace(mapset.into());
        }

        Ok(map)
    }

    pub async fn insert_beatmap(&self, map: &Beatmap) -> InsertMapResult<bool> {
        if should_not_be_stored(map) {
            return Ok(false);
        }

        let mut conn = self.pool.acquire().await?;

        insert_map_(&mut conn, map).await.map(|_| true)
    }
}

async fn insert_map_(conn: &mut PgConnection, map: &Beatmap) -> InsertMapResult<()> {
    let max_combo = if map.mode == GameMode::Mania {
        None
    } else if let Some(combo) = map.max_combo {
        Some(combo as i32)
    } else {
        return Err(InsertMapError::MissingCombo(map.mode).into());
    };

    let query = sqlx::query!(
        "INSERT INTO maps (\
            map_id,\
            mapset_id,\
            checksum,\
            version,\
            seconds_total,\
            seconds_drain,\
            count_circles,\
            count_sliders,\
            count_spinners,\
            hp,\
            cs,\
            od,\
            ar,\
            mode,\
            status,\
            last_update,\
            stars,\
            bpm,\
            max_combo,\
            user_id\
        )\
        VALUES\
        ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)\
        ON CONFLICT (map_id) DO NOTHING",
        map.map_id as i32,
        map.mapset_id as i32,
        map.checksum,
        map.version,
        map.seconds_total as i32,
        map.seconds_drain as i32,
        map.count_circles as i32,
        map.count_sliders as i32,
        map.count_spinners as i32,
        map.hp,
        map.cs,
        map.od,
        map.ar,
        map.mode as i16,
        map.status as i16,
        map.last_updated,
        map.stars,
        map.bpm,
        max_combo,
        map.creator_id as i32,
    );

    query
        .execute(&mut *conn)
        .await
        .map_err(InsertMapError::from)?;

    if let Some(ref mapset) = map.mapset {
        insert_mapset_(conn, mapset).await?;
    }

    Ok(())
}

fn insert_mapset_<'a>(
    conn: &'a mut PgConnection,
    mapset: &'a Beatmapset,
) -> BoxFuture<'a, InsertMapResult<()>> {
    let fut = async move {
        let query = sqlx::query!(
            "INSERT INTO mapsets (\
                mapset_id,\
                user_id,\
                artist,\
                title,\
                creator,\
                status,\
                ranked_date,\
                bpm\
            )\
            VALUES\
            ($1,$2,$3,$4,$5,$6,$7,$8)\
            ON CONFLICT (mapset_id) DO NOTHING",
            mapset.mapset_id as i32,
            mapset.creator_id as i32,
            mapset.artist,
            mapset.title,
            mapset.creator_name.as_str(),
            mapset.status as i16,
            mapset.ranked_date,
            mapset.bpm,
        );

        query
            .execute(&mut *conn)
            .await
            .map_err(InsertMapsetError::from)?;

        if let Some(ref maps) = mapset.maps {
            for map in maps {
                insert_map_(conn, map).await?;
            }
        }

        Ok(())
    };

    fut.boxed()
}
