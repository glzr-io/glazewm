using System.Collections.Generic;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class RunWindowRulesCommand : Command
  {
    public Window Window { get; }
    public List<WindowRuleConfig> WindowRules { get; }

    public RunWindowRulesCommand(Window window, List<WindowRuleConfig> windowRules)
    {
      Window = window;
      WindowRules = windowRules;
    }
  }
}
