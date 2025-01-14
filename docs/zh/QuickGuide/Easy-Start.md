1. 用户注册

    1.1 MQTT Broker 启用了用户验证功能，客户端在发布或订阅消息前，必须提供有效的用户名和密码以通过验证。未通过验证的客户端将无法与 Broker 通信。这一功能可以增强系统的安全性，防止未经授权的访问。

     注册用户
    ```console
    % cli-command  mqtt user create --username=testp --password=7355608
    Created successfully!
    ```
     删除用户
    ```console
    % cli-command  mqtt user delete --username=testp
    Deleted successfully!
    ```
    1.2 查看创建的用户

    ```console
    % cli-command  mqtt user list
    +----------+--------------+
    | username | is_superuser |
    +----------+--------------+
    | admin    | true         |
    +----------+--------------+
    | testp    | false        |
    +----------+--------------+
    ```

2. TODO订阅发布

   2.1 订阅消息

   ```console
    % ./cli-command mqtt sub --username=testp --password=7355608 --topic=topic_test1
    Subscribed!
    ```

   2.2 发布消息

   ```console
    % ./cli-command mqtt pub --username=testp --password=7355608 --topic=topic_test1 --message=HelloWorld!
    Published!
    ```


3. 开启慢订阅功能

   3.1 慢订阅统计功能主要是为了在消息到达Broker后，Broker来计算完成消息处理以及传输整个流程所消耗的时间(时延), 如果时延超过阈值，我们就会记录一条相关的信息在集群慢订阅日志当中，运维人员可以通过命令查询整个集群下的慢订阅记录信息，通过慢订阅信息来解决。

   开启慢订阅
   ```console
    % ./cli-command mqtt slow-sub --enable=true
    The slow subscription feature has been successfully enabled.
   ```

   3.2 如何查看慢订阅记录

    当我们启动了慢订阅统计功能之后, 集群就开启慢订阅统计功能，这样我们可以通过对应的命令来去查询对应的慢订阅记录， 如果我们想要查看慢订阅记录，客户端可以输入如下命令

   ```console
    % ./cli-command mqtt slow-sub --query=true
    +-----------+-------+----------+---------+-------------+
    | client_id | topic | sub_name | time_ms | create_time |
    +-----------+-------+----------+---------+-------------+
    ```

   3.3 排序功能

   如果想要获取更多的慢订阅记录， 并且想要按照从小到大的顺序进行升序排序， 那么可以使用如下的命令

   ```console
    % ./cli-command mqtt slow-sub --list=200 --sort=asc
    +-----------+-------+----------+---------+-------------+
    | client_id | topic | sub_name | time_ms | create_time |
    +-----------+-------+----------+---------+-------------+
    ```

   3.4 筛选查询功能

    对于慢订阅查询，我们同样支持筛选查询功能，我们支持使用topic, sub_name以及client_id的方式来获取不同字段过滤后的结果，其结果默认从大到小倒序排序，参考使用命令如下

    ```console
    % ./cli-command mqtt slow-sub --topic=topic_test1 --list=200
    +-----------+-------+----------+---------+-------------+
    | client_id | topic | sub_name | time_ms | create_time |
    +-----------+-------+----------+---------+-------------+
    ```
