using System.Collections.Generic;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.UserConfigs.Commands
{
  public class RunWithSubjectContainerCommand : Command
  {
    public IEnumerable<string> CommandStrings { get; }
    public Container SubjectContainer { get; }

    public RunWithSubjectContainerCommand(
      IEnumerable<string> commandStrings,
      Container subjectContainer)
    {
      CommandStrings = commandStrings;
      SubjectContainer = subjectContainer;
    }

    public RunWithSubjectContainerCommand(IEnumerable<string> commandStrings)
    {
      CommandStrings = commandStrings;
    }
  }
}
