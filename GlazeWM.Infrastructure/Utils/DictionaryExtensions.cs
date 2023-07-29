using System;
using System.Collections.Generic;

namespace GlazeWM.Infrastructure.Utils
{
  public static class DictionaryExtensions
  {
    public static TValue GetValueOrDefault<TKey, TValue>(
      this Dictionary<TKey, TValue> dictionary,
      TKey key,
      TValue defaultValue)
    {
      TValue val;

      if (dictionary.TryGetValue(key, out val))
        return val ?? defaultValue;

      return defaultValue;
    }

    public static TValue GetValueOrThrow<TKey, TValue>(
      this Dictionary<TKey, TValue> dictionary,
      TKey key)
    {
      TValue val;

      if (!dictionary.TryGetValue(key, out val))
        throw new Exception($"Dictionary value does not exist at key '{key}'.");

      return val;
    }
  }
}
