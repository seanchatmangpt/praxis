use phf::phf_map;

pub enum VarType {
    Request,
    Response,
    Steps,
    Workflows,
    Components,
    SourceDescriptions,
    Url,
    Method,
    StatusCode,
}

pub static ARAZZO_VARS: phf::Map<&'static str, VarType> = phf_map! {
    "$request.header" => VarType::Request,
    "$request.query" => VarType::Request,
    "$request.path" => VarType::Request,
    "$request.body" => VarType::Request,
    "$response.header" => VarType::Response,
    "$response.body" => VarType::Response,
    "$steps" => VarType::Steps,
    "$workflows" => VarType::Workflows,
    "$components" => VarType::Components,
    "$sourceDescriptions" => VarType::SourceDescriptions,
    "$url" => VarType::Url,
    "$method" => VarType::Method,
    "$statusCode" => VarType::StatusCode,
};
