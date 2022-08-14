using System.Linq;
using System.Text.Json;

namespace GlazeWM.Infrastructure.Yaml
{
  public class SnakeCaseNamingPolicy : JsonNamingPolicy
  {
    public static SnakeCaseNamingPolicy Instance { get; } = new SnakeCaseNamingPolicy();

    public override string ConvertName(string name)
    {
      return ToSnakeCase(name);
    }

    public static string ToSnakeCase(string input)
    {
      var snakeCaseLetters = input.Select((letter, index) =>
        index > 0 && char.IsUpper(letter)
          ? "_" + letter.ToString()
          : letter.ToString()
      );

      return string.Concat(snakeCaseLetters).ToLower();
    }
  }
}
