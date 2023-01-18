using System;

namespace GlazeWM.Domain.Common.Enums
{
  public enum Direction
  {
    Up,
    Down,
    Left,
    Right,
  }

  public static class DirectionExtensions
  {
    /// <summary>
    /// Get the inverse of a given direction (eg. `Direction.UP` is the inverse of `Direction.DOWN`).
    /// </summary>
    /// <exception cref="ArgumentOutOfRangeException"></exception>
    public static Direction Inverse(this Direction direction)
    {
      return direction switch
      {
        Direction.Up => Direction.Down,
        Direction.Down => Direction.Up,
        Direction.Left => Direction.Right,
        Direction.Right => Direction.Left,
        _ => throw new ArgumentOutOfRangeException(nameof(direction)),
      };
    }

    /// <summary>
    /// Get the layout that is needed for when moving or switching focus in given direction (eg. a
    /// horizontal layout when moving horizontally).
    /// </summary>
    public static Layout GetCorrespondingLayout(this Direction direction)
    {
      return (direction is Direction.Left or Direction.Right)
        ? Layout.Horizontal
        : Layout.Vertical;
    }
  }
}
