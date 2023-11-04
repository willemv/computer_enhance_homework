/* ========================================================================
   LISTING 65
   ======================================================================== */


fn square(a: f64) -> f64
{
    a * a
}

// NOTE(casey): EarthRadius is generally expected to be 6372.8
pub fn reference_haversine(x0: f64, y0: f64, x1: f64, y1: f64, earth_radius: f64) -> f64
{
    /* NOTE(casey): This is not meant to be a "good" way to calculate the Haversine distance.
       Instead, it attempts to follow, as closely as possible, the formula used in the real-world
       question on which these homework exercises are loosely based.
    */

    let lat1 = y0;
    let lat2 = y1;
    let lon1 = x0;
    let lon2 = x1;

    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();

    let a = square((delta_lat / 2.0).sin()) + (lat1.cos()) * (lat2.cos()) * square((delta_lon / 2.0).sin());
    let c = 2.0 * ((a).sqrt().asin());

    earth_radius * c
}