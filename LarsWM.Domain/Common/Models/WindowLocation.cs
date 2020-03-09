namespace LarsWM.Domain.Common.Models
{
    public class WindowLocation
    {
        public int X { get; private set;}
        public int Y { get; private set;}
        public int Width { get; private set;}
        public int Height { get; private set;}

        public WindowLocation(int x, int y, int width, int height)
        {
            X = x;
            Y = y;
            Width = width;
            Height = height;
        }
    }
}
