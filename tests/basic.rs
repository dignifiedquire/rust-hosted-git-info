extern crate hosted_git_info;

use hosted_git_info::hosted_git::*;

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
