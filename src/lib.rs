#[macro_use] extern crate serde;

mod flextable;
mod datapoint;
mod flexdatavector;
mod flexseries;
mod series;
mod flexdatapoint;
mod globals;

pub use self::flexseries::FlexSeries;
pub use self::flextable::FlexTable;
pub use self::datapoint::DataPoint;
pub use self::flexdatavector::FlexDataVector;
pub use self::series::Series;
pub use self::flexdatapoint::FlexDataPoint;
pub use self::globals::{FlexDataType, FlexData, FlexIndex, FlexIndexType};