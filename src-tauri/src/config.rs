use super::structs::ConfigType;

pub const WX_DLL_NAME: &str = "Weixin.dll";
pub const WX_EXE_NAME: &str = "Weixin.exe";
pub const NEW_WX_EXE_NAME: &str = "Weixin#.exe";
pub const NEW_WX_DLL_NAME: &str = "Weixin.dl#";
pub const WX_DLL_BAK_NAME: &str = "Weixin.dll.bak";
pub const WX_EXE_BAK_NAME: &str = "Weixin.exe.bak";

//ConfigType
//0 版本
//1 dll or exe
//2 原始字节码
//3 替换字节码
//4 共存时是否必须
//5 共存替换
//6 是否搜索状态

pub const UNLOCK:[ConfigType; 3] =[
    (
        "4.0.3",
        "dll",
        "555657534881ECC8010000488DAC248000000048C78540010000FEFFFFFF48C785A800000000000000B960000000",
        "C3...",
        true,
        false,
        true
    ),
    (
        "4.0.2",
        "dll",
        "554157415641545657534881ECD0010000488DAC248000000048C78548010000FEFFFFFF48C7451800000000B960000000",
        "C3...",
        true,
        false,
        true
    ),
    (
        "4.0.0",
        "dll",
        "C74424??FFFFFFFF31F64531C041B9FFFFFFFFFF15????????85C0750F",
        "...EB0F",
        false,
        false,
        true
        )
];

pub const REVOKE: [ConfigType; 3] = [
    (
        "4.0.3",
        "dll",
        "EB??488D8D000?0000E8????????84C0746E",
        "...EB6E",
        false,
        false,
        true,
    ),
    (
        "4.0.2",
        "dll",
        "4885C90F849D010000F0FF41??488B78??E992010000488D8DD0030000E8????????84C0746E",
        "...EB6E",
        false,
        false,
        true,
    ),
    (
        "4.0.0",
        "dll",
        "752148B87265766F6B656D73488905????????66C705????????6700C605????????01488D3D",
        "EB21...",
        false,
        false,
        true,
    ),
];

//打补丁时候 需要修改FF 为 num_u8
pub const CONFIG: [ConfigType; 1] = [(
    "4.0.0",
    "dll",
    "48B8676C6F62616C5F63488905????????C705????????6F6E666966C705????????6700",
    "...C705????????6F6E66##66C705????????6700",
    true,
    true,
    true,
)];

// Redirect host-redirect.xml -> host-redirect.xm1
//打补丁时候 最后一位 修改为 num_u8

pub const HOST: [ConfigType; 1] = [(
    "4.0.0",
    "dll",
    "686F73742D72656469726563742E786D6C",
    "...##",
    true,
    true,
    true,
)];

//1.0.2 Redirect lock.ini -> lock.in1
//打补丁时候 最后一位 修改为 num_u8
//1.0.3 4.0.2版本不需要了
pub const LOCKINI: [ConfigType; 2] = [
    ("4.0.2", "dll", "", "", false, false, true),
    (
        "4.0.0",
        "dll",
        "6C006F0063006B002E0069006E0069",
        "...##",
        true,
        true,
        true,
    ),
];

//Redirect Weixin.dll -> Weixin.dl1
//打补丁时候 最后一位 修改为 num_u8
pub const DLLNAME: [ConfigType; 1] = [(
    "4.0.0",
    "exe",
    "570065006900780069006E002E0064006C006C",
    "...##",
    true,
    true,
    true,
)];

