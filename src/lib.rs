extern crate regex;
extern crate url;

use regex::Regex;
use url::Url;

#[derive(Debug, PartialEq)]
enum Protocol {
    GitSsh,
    GitHttps,
    GitHttp,
    Ssh,
    Https,
    Http,
    Git,
    Shortcut,
}

impl Protocol {
    fn from_str(proto: &str) -> Result<Protocol, ()> {
        if is_shortcut(proto) {
            return Ok(Protocol::Shortcut);
        }

        match proto {
            "git+ssh"   => Ok(Protocol::GitSsh),
            "git+https" => Ok(Protocol::GitHttps),
            "git+http"  => Ok(Protocol::GitHttp),
            "ssh"       => Ok(Protocol::Ssh),
            "https"     => Ok(Protocol::Https),
            "http"      => Ok(Protocol::Http),
            "git"       => Ok(Protocol::Git),
            _           => Err(()),
        }
    }

    fn to_str(&self) -> &str {
        match *self {
            Protocol::GitSsh   => "sshurl",
            Protocol::GitHttps => "https",
            Protocol::GitHttp  => "git+http",
            Protocol::Ssh      => "sshurl",
            Protocol::Https    => "https",
            Protocol::Http     => "http",
            Protocol::Git      => "git",
            Protocol::Shortcut => "shortcut",
        }
    }
}

#[derive(Debug, PartialEq)]
enum Host {
    Github,
    Bitbucket,
    Gitlab,
    Gist,
}

impl Host {
    fn into_iter() -> ::std::vec::IntoIter<Host> {
        vec![
            Host::Github,
            Host::Bitbucket,
            Host::Gitlab,
            Host::Gist,
        ].into_iter()
    }

    fn to_str(&self) -> &str {
        match *self {
            Host::Github    => "github",
            Host::Bitbucket => "bitbucket",
            Host::Gitlab    => "gitlab",
            Host::Gist      => "gist",
        }
    }

    fn to_string(&self) -> String {
        self.to_str().to_string()
    }

    fn from_str(host: &str) -> Result<Host, ()> {
        match host {
            "github"    => Ok(Host::Github),
            "bitbucket" => Ok(Host::Bitbucket),
            "gitlab"    => Ok(Host::Gitlab),
            "gist"      => Ok(Host::Gist),
            _           => Err(()),
        }
    }

    fn from_domain(domain: &str) -> Result<Host, ()> {
        match domain {
            "github.com"      => Ok(Host::Github),
            "bitbucket.org"   => Ok(Host::Bitbucket),
            "gitlab.com"      => Ok(Host::Gitlab),
            "gist.github.com" => Ok(Host::Gist),
            _                 => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HostedGit {
    host: Host,
    user: String,
    password: Option<String>,
    project: String,
    committish: Option<String>,
    protocol: Protocol,
}

impl HostedGit {
    fn new<S>(url: S) -> HostedGit where S: Into<String> {
        let url = url.into();
        if is_git_url(&url) {
            return parse_git_url(&url);
        }

        parse_regular_url(&url)
    }

    fn get_default_representation (&self) -> &str {
        self.protocol.to_str()
    }
}

fn git_re () -> Regex {
    Regex::new(r"^([^@]+)@([^:]+):[/]?((?:[^/]+[/])?[^/]+?)(?:[.]git)?(#.*)?$").unwrap()
}

fn is_git_url (url: &str) -> bool {
    git_re().is_match(url)
}

fn get_shortcut_host (protocol: &str) -> Option<Host> {
    Host::into_iter()
    .filter(|h| h.to_str() == protocol)
    .nth(0)
}

fn is_shortcut (protocol: &str) -> bool {
    get_shortcut_host(protocol).is_some()
}

fn parse_git_url (url: &str) -> HostedGit {
    let cap = git_re().captures(url).unwrap();
    let (user, password) = parse_auth(cap.at(1).unwrap_or(""));

    HostedGit {
        host: cap.at(2).map(|h| Host::from_domain(h).unwrap()).unwrap(),
        user: user.to_string(),
        password: password.map(|p| p.to_string()),
        project: path_to_project(cap.at(3).unwrap_or("/")),
        committish: hash_to_committish(cap.at(4)),
        protocol: Protocol::GitSsh,
    }
}

fn parse_auth<'a> (raw: &'a str) -> (&'a str, Option<&'a str>) {
    let re = Regex::new(r"([^:]+):(.*)").unwrap();

    match re.captures(raw) {
        Some(auth_cap) => {
            (auth_cap.at(0).unwrap(), auth_cap.at(1))
        },
        None => ("", None),
    }
}

fn path_to_project(path: &str) -> String {
    let re = Regex::new(r"^[/](.*?)(?:[.]git)?$").unwrap();
    re.replace(path, "$1")
}

fn hash_to_committish<'a> (hash: Option<&'a str>) -> Option<String> {
    hash.map(|c| (&c[1..]).to_string())
}

fn parse_regular_url (url: &str) -> HostedGit {
    let parsed = Url::parse(url).unwrap();
    let protocol = Protocol::from_str(parsed.scheme()).unwrap();

    let host = if protocol == Protocol::Shortcut {
        get_shortcut_host(parsed.scheme()).unwrap()
    } else {
        parsed.host_str().map(|h| Host::from_domain(h).unwrap()).unwrap()
    };

    let user = parsed.username();
    let password = parsed.password();

    HostedGit {
        host: host,
        user: user.to_string(),
        password: password.map(|p| p.to_string()),
        project: path_to_project(parsed.path()),
        committish: hash_to_committish(parsed.query()),
        protocol: protocol,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(
            HostedGit::new("https://github.com/abc/def").get_default_representation(),
            "https"
        );
        assert_eq!(
            HostedGit::new("ssh://git@github.com/abc/def").get_default_representation(),
            "sshurl"
        );
        assert_eq!(
            HostedGit::new("git+ssh://git@github.com/abc/def").get_default_representation(),
            "sshurl"
        );
        assert_eq!(
            HostedGit::new("git+https://github.com/abc/def").get_default_representation(),
            "https"
        );
        assert_eq!(
            HostedGit::new("git@github.com:abc/def").get_default_representation(),
            "sshurl"
        );
        assert_eq!(
            HostedGit::new("git://github.com/abc/def").get_default_representation(),
            "git"
        );
        assert_eq!(
            HostedGit::new("github:abc/def").get_default_representation(),
            "shortcut"
        );
    }
}
