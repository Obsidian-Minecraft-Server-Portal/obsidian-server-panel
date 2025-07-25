import {LogView} from "./LogView.tsx";
import {Button, Input} from "@heroui/react";
import {Icon} from "@iconify-icon/react";
import {Tooltip} from "../../extended/Tooltip.tsx";


type ServerOverviewProps = {
    id: string;
}

export default function ServerConsole(props: ServerOverviewProps)
{
    const {id} = props;
    return (
        <div className={"flex flex-col gap-2 p-4 bg-default-50 max-h-[calc(100dvh_-_400px)] relative"}>
            <h1 className={"text-4xl mb-4"}>Console</h1>
            <LogView log={log + log}/>
            <div className={"absolute bottom-8 left-8 right-8 font-minecraft-body"}>
                <Input
                    placeholder={"Send a command..."}
                    radius={"none"}
                    startContent={<Icon icon={"mdi:console"}/>}
                    endContent={
                        <Tooltip content={"Send Command"}>
                            <Button isIconOnly variant={"light"} size={"sm"} radius={"none"}><Icon icon={"mdi:send"}/></Button>
                        </Tooltip>
                    }
                />
            </div>
        </div>
    );
}


const log = `
                [00:01:47] [Netty Epoll Acceptor IO Thread/ERROR]: Error sending packet clientbound/minecraft:disconnect
io.netty.handler.codec.EncoderException: Sending unknown packet 'clientbound/minecraft:disconnect'
\tat knot/net.minecraft.class_9136.method_56426(class_9136.java:50) ~[server-intermediary.jar:?]
\tat knot/net.minecraft.class_9136.encode(class_9136.java:14) ~[server-intermediary.jar:?]
\tat knot/net.minecraft.class_2545.method_10838(class_2545.java:26) ~[server-intermediary.jar:?]
\tat knot/net.minecraft.class_2545.encode(class_2545.java:12) ~[server-intermediary.jar:?]
\tat knot/io.netty.handler.codec.MessageToByteEncoder.write(MessageToByteEncoder.java:107) ~[netty-codec-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.invokeWrite0(AbstractChannelHandlerContext.java:893) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.invokeWrite(AbstractChannelHandlerContext.java:875) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.write(AbstractChannelHandlerContext.java:984) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.write(AbstractChannelHandlerContext.java:868) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.ChannelOutboundHandlerAdapter.write(ChannelOutboundHandlerAdapter.java:113) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/net.minecraft.class_2535$2.write(class_2535.java:525) ~[server-intermediary.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.invokeWrite0(AbstractChannelHandlerContext.java:893) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.invokeWriteAndFlush(AbstractChannelHandlerContext.java:956) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.write(AbstractChannelHandlerContext.java:982) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.writeAndFlush(AbstractChannelHandlerContext.java:950) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.writeAndFlush(AbstractChannelHandlerContext.java:1000) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.DefaultChannelPipeline.writeAndFlush(DefaultChannelPipeline.java:974) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannel.writeAndFlush(AbstractChannel.java:305) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/net.minecraft.class_2535.method_36942(class_2535.java:351) ~[server-intermediary.jar:?]
\tat knot/net.minecraft.class_2535.method_10764(class_2535.java:343) ~[server-intermediary.jar:?]
\tat knot/net.minecraft.class_2535.method_52906(class_2535.java:325) ~[server-intermediary.jar:?]
\tat knot/net.minecraft.class_2535.method_10752(class_2535.java:319) ~[server-intermediary.jar:?]
\tat knot/net.minecraft.class_2535.exceptionCaught(class_2535.java:170) ~[server-intermediary.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.invokeExceptionCaught(AbstractChannelHandlerContext.java:346) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.invokeExceptionCaught(AbstractChannelHandlerContext.java:325) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.fireExceptionCaught(AbstractChannelHandlerContext.java:317) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.DefaultChannelPipeline$HeadContext.exceptionCaught(DefaultChannelPipeline.java:1324) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.invokeExceptionCaught(AbstractChannelHandlerContext.java:346) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.AbstractChannelHandlerContext.invokeExceptionCaught(AbstractChannelHandlerContext.java:325) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.DefaultChannelPipeline.fireExceptionCaught(DefaultChannelPipeline.java:856) ~[netty-transport-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.epoll.AbstractEpollStreamChannel$EpollStreamUnsafe.handleReadException(AbstractEpollStreamChannel.java:727) ~[netty-transport-classes-epoll-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.epoll.AbstractEpollStreamChannel$EpollStreamUnsafe.epollInReady(AbstractEpollStreamChannel.java:825) ~[netty-transport-classes-epoll-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.epoll.EpollEventLoop.processReady(EpollEventLoop.java:501) ~[netty-transport-classes-epoll-4.1.118.Final.jar:?]
\tat knot/io.netty.channel.epoll.EpollEventLoop.run(EpollEventLoop.java:399) ~[netty-transport-classes-epoll-4.1.118.Final.jar:?]
\tat knot/io.netty.util.concurrent.SingleThreadEventExecutor$4.run(SingleThreadEventExecutor.java:998) ~[netty-common-4.1.118.Final.jar:?]
\tat knot/io.netty.util.internal.ThreadExecutorMap$2.run(ThreadExecutorMap.java:74) ~[netty-common-4.1.118.Final.jar:?]
\tat java.base/java.lang.Thread.run(Thread.java:1583) [?:?]
[00:07:35] [Netty Epoll Acceptor IO Thread/ERROR]: Error sending packet clientbound/minecraft:disconnect
            `;