extern crate regex;
extern crate url;

use regex::Regex;
use url::Url;


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
            Protocol::GitHttps => "git+https",
            Protocol::GitHttp  => "git+http",
            Protocol::Ssh      => "sshurl",
            Protocol::Https    => "https",
            Protocol::Http     => "http",
            Protocol::Git      => "git",
            Protocol::Shortcut => "shortcut",
        }
    }
}

enum Host {
    Github,
    Bitbucket,
    Gitlab,
    Gist,
}

pub struct HostedGit<'a> {
    host: Host,
    user: &'a str,
    auth: &'a str,
    project: &'a str,
    committish: &'a str,
    protocol: Protocol,
}

impl<'a> HostedGit<'a> {
    fn parse<'b>(url: &'b str) -> HostedGit {
        let parsed = GitUrl::parse(url);

        HostedGit {
            host: Host::Github,
            user: &"hello",
            auth: &"world",
            project: &"cool",
            committish: &"master",
            protocol: parsed.protocol,
        }
    }

    fn get_default_representation (&self) -> &str {
        self.protocol.to_str()
    }
}

struct GitUrl<'a> {
    protocol: Protocol,
    host: &'a str,
    auth: Option<&'a str>,
    port: Option<u16>,
    hash: Option<&'a str>,
    path: &'a str,
}

impl<'a> GitUrl<'a> {
    fn parse (url: &str) -> GitUrl {
        let re = Regex::new(r"^([^@]+)@([^:]+):[/]?((?:[^/]+[/])?[^/]+?)(?:[.]git)?(#.*)?$").unwrap();

        if re.is_match(url) {
            let cap = re.captures(url).unwrap();

            GitUrl {
                protocol: Protocol::GitSsh,
                host: cap.at(2).unwrap(),
                auth: cap.at(1),
                port: None,
                hash: cap.at(4),
                path: ("/".to_string() + cap.at(3).unwrap()).as_ref(),
            }
        } else {
            let parsed = Url::parse(url).unwrap();
            let auth = match parsed.password() {
                Some(pw) => Some((parsed.username().to_string() + pw).as_ref()),
                None     => None
            };

            GitUrl {
                protocol: Protocol::from_str(parsed.scheme()).unwrap(),
                host: parsed.host_str().unwrap(),
                auth: auth,
                port: parsed.port(),
                hash: parsed.query(),
                path: parsed.path(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(HostedGit::parse("https://github.com/abc/def").get_default_representation(), "https");
        assert_eq!(HostedGit::parse("ssh://git@github.com/abc/def").get_default_representation(), "sshurl");
        assert_eq!(HostedGit::parse("git+ssh://git@github.com/abc/def").get_default_representation(), "sshurl");
        assert_eq!(HostedGit::parse("git+https://github.com/abc/def").get_default_representation(), "https");
        assert_eq!(HostedGit::parse("git@github.com:abc/def").get_default_representation(), "sshurl");
        assert_eq!(HostedGit::parse("git://github.com/abc/def").get_default_representation(), "git");
        assert_eq!(HostedGit::parse("github:abc/def").get_default_representation(), "shortcut");
    }
}
