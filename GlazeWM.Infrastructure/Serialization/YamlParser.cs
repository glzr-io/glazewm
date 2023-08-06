using System.IO;
using System.Text.Json;
using YamlDotNet.Core;
using YamlDotNet.Serialization;

namespace GlazeWM.Infrastructure.Serialization
{
  public static class YamlParser
  {
    /// <summary>
    /// The YAML deserialization library doesn't have support for polymorphic objects. Because of
    /// this, the YAML is first converted into JSON and then deserialized via `System.Text.Json`.
    /// </summary>
    public static T ToInstance<T>(string input, JsonSerializerOptions deserializeOptions)
    {
      // Deserializes YAML into key-value pairs (ie. not an object of type `T`). Merging parser is
      // used to enable the use of merge keys.
      var reader = new MergingParser(new Parser(new StringReader(input)));
      var yamlObject = new DeserializerBuilder().Build().Deserialize(reader);

      // Convert key-value pairs into a JSON string.
      var jsonString = JsonParser.ToString(yamlObject);

      return JsonParser.ToInstance<T>(jsonString, deserializeOptions);
    }
  }
}
