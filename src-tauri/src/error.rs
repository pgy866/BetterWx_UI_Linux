use thiserror::Error;

// 自定义错误类型的定义
#[derive(Error, Debug)]
pub enum MyError {
    #[error("读写文件失败,请用管理员模式运行,或解除占用")]
    IoError(#[from] std::io::Error),
    #[error("正则错误")]
    RegError(#[from] regex::Error),
    #[error("序列化错误")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("补丁数据无效")]
    FixPatchDataError,
    #[error("特征码错误")]
    PatternError,
    #[error("不支持的版本")]
    UnSpuuortVersion,
    #[error("读取文件失败")]
    ReadFileError,
    #[error("微信地址错误")]
    WXPathError,
    #[error("搜索{0}地址失败")]
    SearchPatchLocError(String),
    #[error("等待初始化")]
    NeedInitFirst,
    #[error("保存文件失败,请用管理员模式运行,或解除占用")]
    SaveFileError,
    #[error("获取文件列表失败")]
    ReadDirRrror,
    #[error("运行文件失败")]
    RunAppError,
}

impl serde::Serialize for MyError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
