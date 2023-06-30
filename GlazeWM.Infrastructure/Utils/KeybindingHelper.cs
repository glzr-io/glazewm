using System;
using System.Collections.Generic;
using System.Linq;
using System.Windows.Forms;

namespace GlazeWM.Infrastructure.Utils
{
  public static class KeybindingHelper
  {
    public static List<Keys> ParseKeybindingString(string keybindingString)
    {
      return keybindingString
        .Split('+')
        .Select(keyString => FormatKeyString(keyString))
        .Select(keyString => ParseKeyString(keyString))
        .ToList();
    }

    private static string FormatKeyString(string keyString)
    {
      var isNumeric = int.TryParse(keyString, out var _);

      return isNumeric ? $"D{keyString}" : keyString;
    }

    private static Keys ParseKeyString(string keyString)
    {
      try
      {
        return Enum.Parse<Keys>(keyString);
      }
      catch (ArgumentException)
      {
        return keyString switch
        {
          "Ctrl" => Keys.Control,
          _ => throw new ArgumentException($"Unknown key '{keyString}'"),
        };
      }
    }
  }
}
