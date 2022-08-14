using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;
using YamlDotNet.Serialization;

namespace GlazeWM.Infrastructure.Yaml
{
  public class YamlDeserializationService
  {
    private readonly IDeserializer _yamlDeserializer = new DeserializerBuilder()
      .Build();

    private readonly ISerializer _jsonSerializer = new SerializerBuilder()
      .JsonCompatible()
      .Build();

    /// <summary>
    /// The YAML deserializing library doesn't have support for polymorphic objects. Because of
    /// this, the YAML is first converted into JSON and then deserialized via `System.Text.Json`.
    /// </summary>
    public T Deserialize<T>(TextReader input, List<JsonConverter> converters)
    {
      var yamlObject = _yamlDeserializer.Deserialize(input);
      var jsonString = _jsonSerializer.Serialize(yamlObject);

      var jsonDeserializerSettings = GetJsonDeserializerOptions(converters);
      return JsonSerializer.Deserialize<T>(jsonString, jsonDeserializerSettings);
    }

    private static JsonSerializerOptions GetJsonDeserializerOptions(
      List<JsonConverter> converters
    )
    {
      var jsonDeserializerOptions = new JsonSerializerOptions()
      {
        // TODO: Use built-in snake case policy once support is added.
        // Ref: https://github.com/dotnet/runtime/issues/782
        PropertyNamingPolicy = new SnakeCaseNamingPolicy(),
        NumberHandling = JsonNumberHandling.AllowReadingFromString,
        IncludeFields = true,
      };

      foreach (var converter in converters)
        jsonDeserializerOptions.Converters.Add(converter);

      // Enable strings to be mapped to a C# enum (eg. `BarPosition` enum).
      jsonDeserializerOptions.Converters.Add(new JsonStringEnumConverter());

      return jsonDeserializerOptions;
    }
  }
}
