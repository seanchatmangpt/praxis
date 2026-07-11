{application,arazzo_runner,
             [{description,"Arazzo Runner - Workflow Execution Engine wrapping air_core"},
              {vsn,"0.1.0"},
              {registered,[arazzo_runner_sup]},
              {applications,[kernel,stdlib]},
              {mod,{arazzo_runner_app,[]}},
              {env,[]},
              {modules,[arazzo_runner_app,arazzo_runner_sup,
                        arazzo_runner_workflow]}]}.
