#[cfg(test)]
mod tests {
    extern crate torrent_search;
    use torrent_search::{search_l337x, TorrentSearchResult, TorrentSearchError};

    #[test]
    fn search_test() {
        assert_eq!(search_l337x("Debian-8-7-1-Jessie-KDE-x32-i386-CD1-ISO-Uzerus".to_string()).unwrap()[0],
                   TorrentSearchResult {
                       name: "Debian-8-7-1-Jessie-KDE-x32-i386-CD1-ISO-Uzerus".to_string(),
                       magnet: Ok("magnet:?xt=urn:btih:9EB579CA38807332ECA53358E5E014CAD70C1358&dn=Debian+8.7.1+%5BJessie%5D%5BKDE%5D%5Bx32%5D%5Bi386%5D%5BCD1%5D%5BISO%5D%5BUzerus%5D&tr=udp%3A%2F%2Ftracker.zer0day.to%3A1337%2Fannounce&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fcoppersurfer.tk%3A6969%2Fannounce".to_string()),
                   }
        );

        assert_eq!(search_l337x("Arch-Linux-2014-10-10-x86-x64".to_string()).unwrap()[0],
                   TorrentSearchResult {
                       name: "Arch-Linux-2014-10-10-x86-x64".to_string(),
                       magnet: Ok("magnet:?xt=urn:btih:FF71F60D489A634C0E55972A60A50FE7B13A4A4F&dn=Arch+Linux+-+2014.10.10+-+%28x86%2Fx64%29&tr=http%3A%2F%2Ftracker.archlinux.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.zer0day.to%3A1337%2Fannounce&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fcoppersurfer.tk%3A6969%2Fannounce".to_string()),
                   }
        );

    }

    fn invalid_search_test() {
        assert_eq!(search_l337x("dsfadsmfoaisdmvapedoejdapoae".to_string()), Err(TorrentSearchError::NoSearchResults));
        assert_eq!(search_l337x("iqowejfopqewfcosidmfopasdmfpaoeiwmf".to_string()), Err(TorrentSearchError::NoSearchResults));
        //L337X also doesn't allow searches shorter than 3 characters
        assert_eq!(search_l337x("jj".to_string()), Err(TorrentSearchError::SearchTooShort));
        assert_eq!(search_l337x("hi".to_string()), Err(TorrentSearchError::SearchTooShort));

    }
}
