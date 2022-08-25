/* Customize the options here to suit your needs
 * All the COMMAND_* variables shall refer to an executable program on
 * your computer. Beware.
 */

/* This SHALL support opening both URLs and files from the command line */
pub const COMMAND_BROWSER: &str = "firefox";
pub const COMMAND_IMAGE: &str = "feh";
pub const COMMAND_TELNET: &str = "telnet";

/* Set this to None if you want to display text files directly in
 * the gaufre interface
 */
pub const COMMAND_TEXT: Option<&str> = Some("less");

/* I strongly advise you set this to an *absolute* path, if you don't
 * want to see random files spawning in your current directory each
 * time you summon gaufre.
 *
 * Set this to None if you want to be prompted each time a file is to be
 * saved.
 */
pub const DOWNLOAD_FOLDER: Option<&str> = None;

/* IDK, I thought some people would prefer another prefix */
pub const CMD_PREFIX: char = '/';
