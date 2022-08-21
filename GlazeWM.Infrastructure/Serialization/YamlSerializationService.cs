using System.Collections.Generic;
using System.IO;
using System.Text.Json.Serialization;
using YamlDotNet.Serialization;

namespace GlazeWM.Infrastructure.Serialization
{
  public class YamlSerializationService
  {
    private readonly JsonSerializationService _jsonSerializationService;
    private readonly IDeserializer _yamlDeserializer = new DeserializerBuilder()
      .Build();

    public YamlSerializationService(JsonSerializationService jsonSerializationService)
    {
      _jsonSerializationService = jsonSerializationService;
    }

    /// <summary>
    /// The YAML deserializing library doesn't have support for polymorphic objects. Because of
    /// this, the YAML is first converted into JSON and then deserialized via `System.Text.Json`.
    /// </summary>
    public T Deserialize<T>(string input, List<JsonConverter> converters)
    {
      // Deserializes YAML into key-value pairs (ie. not an object of type `T`).
      var yamlObject = _yamlDeserializer.Deserialize(new StringReader(input));

      // Convert key-value pairs into a JSON string.
      var jsonString = _jsonSerializationService.Serialize(yamlObject, converters);

      return _jsonSerializationService.Deserialize<T>(jsonString, converters);
    }
  }
}
