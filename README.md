# restore_after_delete工具说明：
该工具是一个用于在S3开启版本控制情况下，对删除的文件进行一键恢复的工具

## 安装与编译：
    请使用rust进行编译，编译方法 
    ````
        1. 安装rust，执行curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh 
        2. git clone https://github.com/IAmSomeoneLikeYou/restore_after_delete.git 
        3. cd restore_after_delete && cargo build --release 
        4. 执行文件在./target/release/restore_after_delete
    ```
使用示例： 
    ./target/release/restore_after_delete s3://bucket/prefix
