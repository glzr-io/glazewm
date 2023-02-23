namespace GlazeWM.Domain.UserConfigs
{
  public class WeatherComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Text to display.
    /// </summary>
    public string Text { get; set; } = "Hello world!";
    /// <summary>
    /// Latitude to retreive weather.
    /// </summary>
    public float Latitude { get; set; } = 40.7128f;
    /// <summary>
    /// Longitude to retrieve weather.
    /// </summary>
    public float Longitude { get; set; } = 74.0060f;
    /// <summary>
    /// Longitude to retrieve weather.
    /// </summary>
    public string TemperatureUnit { get; set; } = "celcius";
    /// <summary>
    /// Icon to represet sunny weather.
    /// </summary>
    public string LabelSun { get; set; } = "";
    /// <summary>
    /// 
    /// </summary>
    public string LabelMoon { get; set; } = "";
    /// <summary>
    /// 
    /// </summary>
    public string LabelCloudMoon { get; set; } = "";
    /// <summary>
    /// 
    /// </summary>
    public string LabelCloudSun { get; set; } = "⛅";
    /// <summary>
    /// 
    /// </summary>
    public string LabelCloudMoonRain { get; set; } = "";
    /// <summary>
    /// 
    /// </summary>
    public string LabelCloudSunRain { get; set; } = "";
    /// <summary>
    /// 
    /// </summary>
    public string LabelCloudRain { get; set; } = "";
    /// <summary>
    /// 
    /// </summary>
    public string LabelSnowflake { get; set; } = "❄";
    /// <summary>
    /// 
    /// </summary>
    public string LabelThunderstorm { get; set; } = "";
    /// <summary>
    /// 
    /// </summary>
    public string LabelCloud { get; set; } = "☁";
    /// <summary>
    /// Sets default icon font if one isn't specified.
    /// </summary>
    public WeatherComponentConfig()
    {
      FontFamily = "pack://application:,,,/Resources/#Font Awesome 6 Free Solid";
    }
  }
}
