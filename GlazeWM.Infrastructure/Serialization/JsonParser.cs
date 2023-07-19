using System;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace GlazeWM.Infrastructure.Serialization
{
  public static class JsonParser
  {
    private static readonly JsonSerializerOptions _defaultOptions = new()
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

    public static string ToString<T>(T value)
    {
      return JsonSerializer.Serialize(value, _defaultOptions);
    }

    public static string ToString<T>(T value, JsonSerializerOptions options)
    {
      return JsonSerializer.Serialize(value, options);
    }

    public static T ToInstance<T>(string json)
    {
      return JsonSerializer.Deserialize<T>(json, _defaultOptions);
    }

    public static T ToInstance<T>(string json, JsonSerializerOptions options)
    {
      return JsonSerializer.Deserialize<T>(json, options);
    }

    public static JsonSerializerOptions OptionsFactory(
      Action<JsonSerializerOptions> callback)
    {
      var options = new JsonSerializerOptions(_defaultOptions);
      callback(options);
      return options;
    }
  }
}
