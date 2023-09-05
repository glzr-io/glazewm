using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.UserConfigs.Commands
{
  public class RunWithSubjectContainerCommand : Command
  {
    public List<string> CommandStrings { get; }
    public Container SubjectContainer { get; }

    public RunWithSubjectContainerCommand(
      List<string> commandStrings,
      Container subjectContainer)
    {
      CommandStrings = commandStrings;
      SubjectContainer = subjectContainer;
    }

    public RunWithSubjectContainerCommand(List<string> commandStrings)
    {
      CommandStrings = commandStrings;
    }
  }
}
