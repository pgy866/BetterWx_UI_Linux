use super::structs::ConfigType;

// Linux WeChat file names
pub const WX_LIB_NAME: &str = "libWeixinCore.so";
pub const WX_EXE_NAME: &str = "wechat";
pub const NEW_WX_EXE_NAME: &str = "wechat#";
pub const NEW_WX_LIB_NAME: &str = "libWeixinCore#.so";
pub const WX_LIB_BAK_NAME: &str = "libWeixinCore.so.bak";
pub const WX_EXE_BAK_NAME: &str = "wechat.bak";

// Linux WeChat common installation paths
pub const LINUX_WX_PATHS: &[&str] = &[
    "/opt/wechat-beta",
    "/opt/wechat",
    "/usr/share/wechat",
    "/usr/lib/wechat",
];

// ConfigType
// 0: version
// 1: "lib" or "exe" (lib = .so shared library, exe = wechat binary)
// 2: original byte pattern (hex)
// 3: replacement byte pattern (hex)
// 4: is forced when coexist
// 5: need replace number
// 6: should search status

// Anti-recall patch patterns for Linux WeChat
// The revoke logic in the WeChat binary uses the same core code as Windows.
// The pattern searches for the conditional jump around the revoke handler
// and patches it to skip message deletion, preserving revoke tips.
//
// Strategy: Search for "revokemsg" string reference in the binary,
// then locate the conditional branch that controls whether to process the revoke.
// Patch the conditional jump to unconditional jump to skip revoke processing.
pub const REVOKE: [ConfigType; 3] = [
    (
        "4.0.3",
        "lib",
        "EB??488D8D000?0000E8????????84C0746E",
        "...EB6E",
        false,
        false,
        true,
    ),
    (
        "4.0.2",
        "lib",
        "4885C90F849D010000F0FF41??488B78??E992010000488D8DD0030000E8????????84C0746E",
        "...EB6E",
        false,
        false,
        true,
    ),
    (
        "4.0.0",
        "lib",
        "752148B87265766F6B656D73488905????????66C705????????6700C605????????01488D3D",
        "EB21...",
        false,
        false,
        true,
    ),
];

// Multi-instance unlock patterns for Linux
// Patches the mutex/lock check to allow multiple instances
pub const UNLOCK: [ConfigType; 3] = [
    (
        "4.0.3",
        "lib",
        "555657534881ECC8010000488DAC248000000048C78540010000FEFFFFFF48C785A800000000000000B960000000",
        "C3...",
        true,
        false,
        true,
    ),
    (
        "4.0.2",
        "lib",
        "554157415641545657534881ECD0010000488DAC248000000048C78548010000FEFFFFFF48C7451800000000B960000000",
        "C3...",
        true,
        false,
        true,
    ),
    (
        "4.0.0",
        "lib",
        "C74424??FFFFFFFF31F64531C041B9FFFFFFFFFF15????????85C0750F",
        "...EB0F",
        false,
        false,
        true,
    ),
];

// Config path isolation for coexist (multi-instance)
pub const CONFIG: [ConfigType; 1] = [(
    "4.0.0",
    "lib",
    "48B8676C6F62616C5F63488905????????C705????????6F6E666966C705????????6700",
    "...C705????????6F6E66##66C705????????6700",
    true,
    true,
    true,
)];

// Host redirect isolation for coexist
pub const HOST: [ConfigType; 1] = [(
    "4.0.0",
    "lib",
    "686F73742D72656469726563742E786D6C",
    "...##",
    true,
    true,
    true,
)];

// Lock file isolation for coexist (not needed for 4.0.2+)
pub const LOCKINI: [ConfigType; 2] = [
    ("4.0.2", "lib", "", "", false, false, true),
    (
        "4.0.0",
        "lib",
        "6C006F0063006B002E0069006E0069",
        "...##",
        true,
        true,
        true,
    ),
];

// Library name redirect for coexist
// On Linux we redirect the .so name instead of .dll
pub const LIBNAME: [ConfigType; 1] = [(
    "4.0.0",
    "exe",
    "6C696257656978696E436F72652E736F",
    "...##",
    true,
    true,
    true,
)];
