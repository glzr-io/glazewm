namespace LarsWM.Infrastructure.Bussing
{
    public class Command
    {
        public string Name { get; set; }

        public Command()
        {
            Name = GetType().Name;
        }
    }
}
