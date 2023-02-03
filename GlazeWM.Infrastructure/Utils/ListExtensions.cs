using System.Collections.Generic;

namespace GlazeWM.Infrastructure.Utils
{
  public static class ListExtensions
  {
    /// <summary>
    /// Replace the first occurrence of a value with a new value in a list.
    /// </summary>
    public static void Replace<T>(this IList<T> source, T oldValue, T newValue)
    {
      var index = source.IndexOf(oldValue);

      if (index != -1)
        source[index] = newValue;
    }

    /// <summary>
    /// Shift a value to a specified index in a list. Insert at index if it doesn't already exist.
    /// </summary>
    public static void ShiftToIndex<T>(this IList<T> source, int index, T value)
    {
      var initialIndex = source.IndexOf(value);

      if (initialIndex != -1)
        source.Remove(value);

      source.Insert(index, value);
    }

    /// <summary>
    /// Shift a value to the first index of a list. Insert at start if it doesn't already exist.
    /// </summary>
    public static void MoveToFront<T>(this IList<T> source, T value)
    {
      ShiftToIndex(source, 0, value);
    }

    /// <summary>
    /// Insert a value between each element of a list.
    /// </summary>
    public static void Intersperse<T>(this IList<T> source, T value)
    {
      for (var i = 1; i < source.Count; i += 2)
      {
        source.Insert(i, value);
      }
    }
  }
}
