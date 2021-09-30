using System;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace GlazeWM.Domain.UserConfigs
{
  public class BarComponentConfigConverter : JsonConverter
  {
    public override bool CanConvert(Type type) => type.IsAssignableFrom(typeof(BarComponentConfig));

    public override object ReadJson(JsonReader reader, Type objectType, object existingValue, JsonSerializer serializer)
    {
      var jObject = JObject.Load(reader);

      // Get the type of workspace component config.
      var type = jObject["Type"].Value<string>();

      object target = type switch
      {
        "workspaces" => new WorkspacesComponentConfig(),
        "clock" => new ClockComponentConfig(),
        _ => throw new ArgumentException(),
      };

      serializer.Populate(jObject.CreateReader(), target);

      return target;
    }

    /// <summary>
    /// Serializing is not needed, so it's fine to leave it unimplemented.
    /// </summary>
    public override void WriteJson(JsonWriter writer, object value, JsonSerializer serializer)
    {
      throw new NotImplementedException();
    }
  }
}
