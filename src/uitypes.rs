use serde_derive::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct SwitchPort {
    pub speed:      String,
    pub link:       String,
    pub gateway:    bool,
}

#[derive(Serialize)]
pub struct IpRoute {
    pub via: String,
    pub src: String,
}

#[derive(Serialize)]
pub struct Net {
    pub switch:     HashMap<String, SwitchPort>,
    pub iproute:    HashMap<String, IpRoute>,
}



#[derive(Serialize)]
pub struct Station{
    pub vendor:         Option<String>,
    pub mac:            String,
    pub ip:             Option<String>,
    pub host_name:      Option<String>,
    pub radio:          Option<String>,
    pub signal:         Option<i32>,
    #[serde(rename="signal avg")]
    pub signal_avg:     Option<i32>,
}


#[derive(Serialize, Default)]
pub struct Stations{
    pub public:    Vec<Station>,
    pub private:   Vec<Station>,
    pub grid:      Vec<Station>,
    pub access:    Vec<Station>,
    pub thing:     Vec<Station>,
    pub other:     Vec<Station>,
}
