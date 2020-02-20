namespace LarsWM.Domain.Common.Models
{
    public enum Orientation
    {
        Vertical,
        Horizontal,
    }

    public class SplitContainer : Container
    {
        public Orientation Orientation { get; set; }
    }
}
