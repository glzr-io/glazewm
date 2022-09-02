using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace GlazeWM.Infrastructure.Serialization
{
  public class JsonService
  {
    private readonly JsonSerializerOptions _jsonSerializerOptions = new()
    {
      // TODO: Use built-in snake case policy once support is added.
      // Ref: https://github.com/dotnet/runtime/issues/782
      PropertyNamingPolicy = new SnakeCaseNamingPolicy(),
      NumberHandling = JsonNumberHandling.AllowReadingFromString,
      IncludeFields = true,
      Converters = {
        // Enable strings to be mapped to a C# enum (eg. `BarPosition` enum).
        new JsonStringEnumConverter(),
        // Enable boolean strings to be mapped to a C# bool (eg. `"true"` -> `true`).
        new JsonStringBoolConverter(),
      }
    };

    public string Serialize<T>(T value, List<JsonConverter> converters)
    {
      var jsonSerializerOptions = GetJsonSerializerOptions(converters);
      return JsonSerializer.Serialize(value, jsonSerializerOptions);
    }

    public T Deserialize<T>(string json, List<JsonConverter> converters)
    {
      var jsonDeserializerOptions = GetJsonSerializerOptions(converters);
      return JsonSerializer.Deserialize<T>(json, jsonDeserializerOptions);
    }

    private JsonSerializerOptions GetJsonSerializerOptions(List<JsonConverter> converters)
    {
      var jsonSerializerOptions = new JsonSerializerOptions(_jsonSerializerOptions);

      foreach (var converter in converters)
        jsonSerializerOptions.Converters.Add(converter);

      return jsonSerializerOptions;
    }
  }
}
