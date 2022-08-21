using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace GlazeWM.Infrastructure.Serialization
{
  public static class JsonSerializationService
  {
    public static string Serialize<T>(T value, List<JsonConverter> converters)
    {
      var jsonSerializerOptions = GetJsonSerializerOptions(converters);
      return JsonSerializer.Serialize(value, jsonSerializerOptions);
    }

    public static T Deserialize<T>(string json, List<JsonConverter> converters)
    {
      var jsonDeserializerOptions = GetJsonSerializerOptions(converters);
      return JsonSerializer.Deserialize<T>(json, jsonDeserializerOptions);
    }

    private static JsonSerializerOptions GetJsonSerializerOptions(
      List<JsonConverter> converters
    )
    {
      var jsonSerializerOptions = new JsonSerializerOptions()
      {
        // TODO: Use built-in snake case policy once support is added.
        // Ref: https://github.com/dotnet/runtime/issues/782
        PropertyNamingPolicy = new SnakeCaseNamingPolicy(),
        NumberHandling = JsonNumberHandling.AllowReadingFromString,
        IncludeFields = true,
      };

      foreach (var converter in converters)
        jsonSerializerOptions.Converters.Add(converter);

      // Enable strings to be mapped to a C# enum (eg. `BarPosition` enum).
      jsonSerializerOptions.Converters.Add(new JsonStringEnumConverter());

      return jsonSerializerOptions;
    }
  }
}
