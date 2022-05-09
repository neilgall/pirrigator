//!
//!
//!


fn test()
{
    client = Influx::connect("influx.voipir.cl");

    let measurement = client.measurement("cmv.scale.weight")
        .resolution(Resolution::Milliseconds)
        .retention(Retention::Weekly);

    measurement.measure()
        .datetime(ChronoUtc::now())
        .tag("scale", "smallbag")
        .tag("scale", "bran")
        .field("weight", 123)
        .commit();

    measurement.measure()
        .tag("scale", "smallbag")
        .field("weight", 234)
        .add()
        .tag("scale", "smallbag")
        .field("weight", 345)
        .add()
        .tag("scale", "bran")
        .field("weight", 456)
        .commit()
}
