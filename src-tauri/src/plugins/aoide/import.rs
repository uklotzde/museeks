use std::borrow::Cow;

use anyhow::anyhow;
use aoide::{
    media::content::ContentMetadata,
    track::{index::Index, tag::FACET_ID_GENRE, Actor},
    util::clock::DateOrDateTime,
};

use crate::{libs::track::{NumberOf, Track}, plugins::database::Playlist};

const DEFAULT_DURATION: u32 = 0;

fn collect_artists(actors: impl IntoIterator<Item = Actor>) -> Vec<String> {
    let (summary_artist, individual_artists) = actors.into_iter().fold(
        (None, Vec::new()),
        |(mut summary_artist, mut individual_artists), actor| {
            if matches!(actor.role, aoide::track::actor::Role::Artist) {
                match actor.kind {
                    aoide::track::actor::Kind::Summary => {
                        debug_assert!(summary_artist.is_none());
                        summary_artist = Some(actor.name);
                    }
                    aoide::track::actor::Kind::Individual => {
                        individual_artists.push(actor.name);
                    }
                    aoide::track::actor::Kind::Sorting => (),
                }
            }
            (summary_artist, individual_artists)
        },
    );
    if individual_artists.is_empty() {
        // Fallback: Use the summary artist if no individual artists are present.
        // This is probably the common case.
        summary_artist.into_iter().collect()
    } else {
        // Prefer individual artists over the single summary artist if available.
        individual_artists
    }
}

fn year(dt: DateOrDateTime) -> Option<u32> {
    dt.year().try_into().ok()
}

fn number_of(index: Index) -> NumberOf {
    NumberOf {
        no: index.number.map(Into::into),
        of: index.total.map(Into::into),
    }
}

#[allow(dead_code)] // TODO
pub(crate) fn import_track_from_entity(
    entity: aoide::TrackEntity,
    missing_title: impl FnOnce() -> anyhow::Result<String>,
) -> anyhow::Result<Track> {
    let uid = entity.raw.hdr.uid;
    let entity_body = entity.raw.body;
    let content_url = entity_body
        .content_url
        .ok_or_else(|| anyhow!("missing content URL"))?;
    let content_file_path = content_url
        .to_file_path()
        .map_err(|()| anyhow!("unsupported content URL: {content_url}"))?;
    let track = entity_body.track;
    let track_title = track
        .titles
        .untie()
        .into_iter()
        .find_map(|title| {
            if matches!(title.kind, aoide::track::title::Kind::Main) {
                Some(title.name)
            } else {
                None
            }
        })
        .map(Ok)
        .unwrap_or_else(|| missing_title())?;
    let track_artists = collect_artists(track.actors.untie());
    let album_title = track
        .album
        .untie()
        .titles
        .untie()
        .into_iter()
        .find_map(|title| {
            if matches!(title.kind, aoide::track::title::Kind::Main) {
                Some(title.name)
            } else {
                None
            }
        })
        .unwrap_or_default();
    let genres = track
        .tags
        .untie()
        .facets
        .into_iter()
        .filter_map(|facet| {
            if facet.facet_id == *FACET_ID_GENRE {
                Some(
                    facet
                        .tags
                        .into_iter()
                        .filter_map(|tag| tag.label)
                        .map(Cow::from)
                        .map(Cow::into_owned),
                )
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<_>>();
    let year = track
        .recorded_at
        .and_then(year)
        .into_iter()
        .chain(track.released_at.and_then(year))
        .chain(track.released_orig_at.and_then(year))
        .min();
    let ContentMetadata::Audio(audio) = track.media_source.content.metadata;
    let duration = audio
        .duration
        .map(|duration| {
            (duration.value() / 1000.0)
                .round()
                .min(DEFAULT_DURATION.into()) as _
        })
        .unwrap_or(DEFAULT_DURATION);
    let track_number_of = number_of(track.indexes.track);
    let disk_number_of = number_of(track.indexes.disc);
    Ok(Track {
        _id: uid.to_string(),
        title: track_title,
        artists: track_artists,
        album: album_title,
        genres,
        year,
        duration,
        track: track_number_of,
        disk: disk_number_of,
        path: content_file_path,
    })
}

#[allow(dead_code)] // TODO
pub(crate) fn import_playlist_from_entity(
    entity: aoide::PlaylistEntity,
) -> anyhow::Result<Playlist> {
    let uid = entity.raw.hdr.uid;
    let entity_body = entity.raw.body;
    let name = entity_body.title;
    Ok(Playlist {
        _id: uid.to_string(),
        name,
        tracks: vec![],
        import_path: None,
    })
}

#[allow(dead_code)] // TODO
pub(crate) fn import_playlist_track_entries<T>(
    entries: impl IntoIterator<Item = T>,
) -> impl Iterator<Item = String>
where
    T: AsRef<aoide::playlist::Entry>,
{
    entries.into_iter().filter_map(|entry| {
        let entry = entry.as_ref();
        match &entry.item {
            aoide::playlist::Item::Track(track) => Some(track.uid.to_string()),
            _ => None,
        }
    })
}
