//!### Usage:
//! To search for a torrent, simply use the search_l337x function
//!
//! ```
//! use torrent_search::{search_l337x, TorrentSearchResult, TorrentSearchError};
//!
//! #[tokio::main]
//! async fn main() {
//! let debian_search_results = search_l337x("Debian ISO".to_string()).await.unwrap();
//!
//! for result in debian_search_results {
//!     println!("Name of torrent: {}\nMagnet: {}\nSeeders: {}\nLeeches: {}", result.name, result.magnet.unwrap(), result.seeders.unwrap(), result.leeches.unwrap());
//! }
//! }
//!
//! ```
//!
//! This will return `Result<Vec<TorrentSearchResult>, TorrentSearchError>`, which when unwrapped
//! gives a Vector of TorrentSearchResults (shocking I know).
//!
//! You can view more information about the data types of the structs [here](struct.TorrentSearchResult.html)
//!
//!
#![deny(missing_docs)]
#![forbid(unsafe_code)]
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

#[macro_use]
extern crate lazy_static;

///Torrent regex str
const TORRENT_RES_RE_STR: &str = "<td class=\"coll-1 name\"><a href=\"/sub/[0-9]*/[0-9]*/\" class=\"icon\"><i class=\"flaticon-[a-zA-Z0-9]*\"></i></a><a href=\"(/torrent/[0-9]*/([a-zA-Z0-9-_+!@#$%^&*()]*))";
const MAGNET_RE_STR: &str = r"(stratum-|)magnet:\?xt=urn:(sha1|btih|ed2k|aich|kzhash|md5|tree:tiger):([A-Fa-f0-9]+|[A-Za-z2-7]+)&[A-Za-z0-9!@#$%^&*=+.\-_()]*(announce|[A-Fa-f0-9]{40}|[A-Za-z2-7]+)";
const SEEDS_RE_STR: &str = "<span class=\"seeds\">([0-9])+</span>";
const LEECHES_RE_STR: &str = "<span class=\"leeches\">([0-9])+</span>";

/// If you get this, that means something went wrong while either scraping or getting the torrent page.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TorrentSearchError {
    /// Returns this if find_torrents fails
    NoSearchResults,
    //While wrapping an error inside another error is annoying, its the only way to give consistent results
    ///ReqwestError converted to a String, since minreq::Error is pretty restrictive
    ReqwestError(String),
    ///If you get this error, it probably means the regex failed.
    MagnetNotFound,
    ///L337X needs searches to be longer than 3 characters. I could've just returned a NoSearchResults
    /// error, but that is more confusing and harder to debug
    SearchTooShort,
    /// The seeds regex failed
    SeedsNotFound,
    /// The leeches regex failed
    LeechesNotFound
}

///This necessary to make using minreq::get possible
impl From<reqwest::Error> for TorrentSearchError {
    fn from(e: reqwest::Error) -> Self {
        TorrentSearchError::ReqwestError(e.to_string())
    }
}

///Some of the basic information of the torrent
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TorrentSearchResult {
    ///The name of the torrent, should be equal to the display name in the magnet url
    pub name: String,
    ///Seeders, of course
    pub seeders: Result<usize, TorrentSearchError>,
    ///Leeches, of course
    pub leeches: Result<usize, TorrentSearchError>,
    ///The magnet url as a string (considered releasing it as a Magnet struct, but decided against it
    ///It's wrapped in a result since the torrent search can work, but accessing a magnet can fail
    pub magnet: Result<String, TorrentSearchError>,

}

///The function takes a search string, then uses web scraping using regex to find the various parts
/// of the search. The search must be longer than 3 characters
pub async fn search_l337x(search: String) -> Result<Vec<TorrentSearchResult>, TorrentSearchError> {
    if search.graphemes(true).count() >= 3 {
        let mut search_results: Vec<TorrentSearchResult> = Vec::new();

        let torrents = find_torrents(get_l337x(search).await?);

        match torrents {
            Ok(torrents) =>
                {
                    for (i, val) in torrents.0.iter().enumerate() {
                        let (seeder_info, leeches_info) = find_peer_info(val).await?;

                        search_results.push(
                            TorrentSearchResult {
                                name: String::from(&torrents.1[i]),
                                magnet: match find_magnet(val).await {
                                    Ok(m) => Ok(m),
                                    Err(e) => Err(e),
                                },
                                seeders: seeder_info,
                                leeches: leeches_info,
                            }
                        );
                    }

                    Ok(search_results)
                },

            Err(e) => {
                Err(e)
            },
        }
    } else {
        Err(TorrentSearchError::SearchTooShort)
    }

}

async fn get_l337x(search: String) -> Result<String, reqwest::Error> {
    //Remove all slashes from searches, as 1337x searches do not allow them
    let page = reqwest::get( &format!("https://1337x.to/search/{}/1/", search.replace("/", "+").replace("%2F", "+").replace("%2f", "+"))).await?.text().await?;
    
    Ok(page)
}

fn find_torrents(page: String) -> Result<(Vec<String>, Vec<String>), TorrentSearchError> {
    lazy_static! {
        static ref TORRENT_RES_RE: Regex = Regex::new(TORRENT_RES_RE_STR).unwrap();
    }


    //Index 0 of the tuple has the torrent url, index 1 has its name
    let responses = {
        let mut responses: (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());

        for result in TORRENT_RES_RE.captures_iter(&page) {
            //Gotta add a slash at the end of the urls, or else it's invalid and will give a 404 if you visit it on 1337x
            responses.0.push(format!("{}{}", result.get(1).map_or("", |m| m.as_str()).to_string(), "/"));
            responses.1.push(result.get(2).map_or("", |m| m.as_str()).to_string());
        }

        responses
    };

    //I only need to check responses.0 since they are both guaranteed to be the same length (or an
    // error will have already occurred)
    if responses.0.len() != 0 {
        Ok(responses)
    } else {
        Err(TorrentSearchError::NoSearchResults)
    }
}

///Scrapes the details page of a torrent for the magnet url
async fn find_magnet(url: &String) -> Result<String, TorrentSearchError> {
    lazy_static! {
        static ref MAGNET_RE: Regex = Regex::new(MAGNET_RE_STR).unwrap();
    }

    let page = reqwest::get( &format!("https://1337x.to{}", url) ).await?.text().await?;

    match MAGNET_RE.captures(&page) {
        Some(captures) => Ok(captures.get(0).map_or("", |m| m.as_str()).to_string()),
        None => Err(TorrentSearchError::MagnetNotFound),
    }
}

async fn find_peer_info(url: &String) -> Result<(Result<usize, TorrentSearchError>, Result<usize, TorrentSearchError>), TorrentSearchError> {
    lazy_static! {
        static ref SEEDS_RE: Regex = Regex::new(SEEDS_RE_STR).unwrap();
        static ref LEECHES_RE: Regex = Regex::new(LEECHES_RE_STR).unwrap();
    }

    let page = reqwest::get( &format!("https://1337x.to{}", url)).await?.text().await?;

    let seeds = match SEEDS_RE.captures(&page) {
        Some(captures) => Ok(captures.get(1).map_or("", |m| m.as_str()).parse::<usize>().unwrap()),
        None => Err(TorrentSearchError::SeedsNotFound),
    };

    let leeches = match LEECHES_RE.captures(&page) {
        Some(captures) => Ok(captures.get(1).map_or("", |m| m.as_str()).parse::<usize>().unwrap()),
        None => Err(TorrentSearchError::LeechesNotFound),
    };

    Ok((seeds, leeches))

}
