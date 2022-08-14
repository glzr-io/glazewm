using System.Collections.Generic;
using System.IO;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace GlazeWM.Infrastructure.Yaml
{
  public class YamlDeserializationService
  {
    private readonly IDeserializer _yamlDeserializer = new DeserializerBuilder()
      .WithNamingConvention(UnderscoredNamingConvention.Instance)
      .Build();

    private readonly ISerializer _jsonSerializer = new SerializerBuilder()
      .JsonCompatible()
      .Build();

    /// <summary>
    /// The YAML deserializing library doesn't have support for polymorphic objects. Because of
    /// this, the YAML is first converted into JSON and then deserialized via `Newtonsoft.Json`.
    /// </summary>
    public T Deserialize<T>(TextReader input, List<JsonConverter> converters)
    {
      var yamlObject = _yamlDeserializer.Deserialize(input);
      var jsonString = _jsonSerializer.Serialize(yamlObject);

      var jsonDeserializerSettings = GetJsonDeserializerSettings(converters);
      return JsonConvert.DeserializeObject<T>(jsonString, jsonDeserializerSettings);
    }

    private static JsonSerializerSettings GetJsonDeserializerSettings(
      List<JsonConverter> converters
    )
    {
      return new()
      {
        MissingMemberHandling = MissingMemberHandling.Error,
        ContractResolver = new DefaultContractResolver
        {
          NamingStrategy = new SnakeCaseNamingStrategy()
        },
        Converters = converters
      };
    }
  }
}
