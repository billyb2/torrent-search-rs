use minreq::get;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

#[macro_use]
extern crate lazy_static;

///Torrent regex str
const TORRENT_RES_RE_STR: &str = "<td class=\"coll-1 name\"><a href=\"/sub/[0-9]*/[0-9]*/\" class=\"icon\"><i class=\"flaticon-[a-zA-Z0-9]*\"></i></a><a href=\"(/torrent/[0-9]*/([a-zA-Z0-9-_+!@#$%^&*()]*))";
const MAGNET_RE_STR: &str = r"(stratum-|)magnet:\?xt=urn:(sha1|btih|ed2k|aich|kzhash|md5|tree:tiger):([A-Fa-f0-9]+|[A-Za-z2-7]+)&[A-Za-z0-9!@#$%^&*=+.\-_()]*(announce|[A-Fa-f0-9]{40}|[A-Za-z2-7]+)";

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TorrentSearchError {
    NoSearchResults,
    //While wrapping an error inside another error is annoying, its the only way to give consistent results
    ///MinreqError converted to a String, since minreq::Error is pretty restrictive
    MinreqError(String),
    ///If you get this error, it probably means the regex failed.
    MagnetNotFound,
    SearchTooShort,
}

///This necessary to make using minreq::get possible
impl From<minreq::Error> for TorrentSearchError {
    fn from(e: minreq::Error) -> Self {
        TorrentSearchError::MinreqError(e.to_string())
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TorrentSearchResult {
    pub name: String,
    pub magnet: Result<String, TorrentSearchError>,

}

///The function takes a search string, then uses web scraping using regex to find the various parts
/// of the search. The search must be longer than 3 characters
pub fn search_l337x(search: String) -> Result<Vec<TorrentSearchResult>, TorrentSearchError> {
    if search.graphemes(true).count() >= 3 {
        let mut search_results: Vec<TorrentSearchResult> = Vec::new();

        let torrents = find_torrents(get_l337x(search)?);

        match torrents {
            Ok(torrents) =>
                {
                    for (i, val) in torrents.0.iter().enumerate() {
                        search_results.push(
                            TorrentSearchResult {
                                name: String::from(&torrents.1[i]),
                                magnet: match find_magnet(val) {
                                    Ok(m) => Ok(m),
                                    Err(e) => Err(e),
                                },
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

fn get_l337x(search: String) -> Result<String, minreq::Error> {
    //Remove all slashes from searches, as 1337x searches do not allow them
    let url = format!("https://1337x.to/search/{}/1/", search.replace("/", "+").replace("%2F", "+").replace("%2f", "+"));
    //Remove the first 5000 bytes, to make the regex run faster (the first 5000 are guaranteed not to
    // contain any links
    let page = get(url).send()?.as_str()?[5000..].to_string();
    Ok(page)
}

///Scrapes the search page for torrents
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
fn find_magnet(url: &String) -> Result<String, TorrentSearchError> {
    lazy_static! {
        static ref MAGNET_RE: Regex = Regex::new(MAGNET_RE_STR).unwrap();
    }

    let page = get( format!("https://1337x.to{}", url)).send()?.as_str()?[5000..].to_string();

    match MAGNET_RE.captures(&page) {
        Some(captures) => Ok(captures.get(0).map_or("", |m| m.as_str()).to_string()),
        None => Err(TorrentSearchError::MagnetNotFound),
    }
}