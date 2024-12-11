#[macro_export] macro_rules! date {
    ($y:literal-$m:literal-$d:literal) => {
        chrono::NaiveDate::from_ymd_opt($y, $m, $d).unwrap()
    };
}

#[macro_export] macro_rules! times {
     ($hs:literal:$ms:literal--$he:literal:$me:literal) => {
         NaiveTimePeriod::from_hm_hm($hs, $ms, $he, $me)
     };
 }
