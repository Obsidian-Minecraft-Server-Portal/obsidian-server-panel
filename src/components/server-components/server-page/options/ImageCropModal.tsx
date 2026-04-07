import {createRef, useEffect, useState} from "react";
import {Button, Modal, ModalBody, ModalDialog, ModalFooter, ModalHeader} from "@heroui/react";
import Cropper, {ReactCropperElement, ReactCropperProps} from "react-cropper";
import "../../../../css/cropper.min.css";

type ImageCropModalProps =
    {
        image: File | null;
        isOpen: boolean;
        onClose: (croppedData: File | null) => void;
    } & ReactCropperProps

export function ImageCropModal(props: ImageCropModalProps)
{
    const cropperRef = createRef<ReactCropperElement>();
    const [image, setImage] = useState<string>(() =>
        props.image ? URL.createObjectURL(props.image) : ""
    );


    useEffect(() =>
    {
        if (props.image) setImage(URL.createObjectURL(props.image));
    }, [props.image]);

    const getCropData = () =>
    {
        if (cropperRef.current && cropperRef.current.cropper)
        {
            const croppedData = cropperRef.current.cropper.getCroppedCanvas().toDataURL();
            // convert base64 to an image file
            const byteString = atob(croppedData.split(",")[1]);
            const mimeString = croppedData.split(",")[0].split(":")[1].split(";")[0];
            const ab = new ArrayBuffer(byteString.length);
            const ia = new Uint8Array(ab);
            for (let i = 0; i < byteString.length; i++)
            {
                ia[i] = byteString.charCodeAt(i);
            }
            props.onClose(new File([new Blob([ab], {type: mimeString})], "server-icon.png", {type: mimeString}));
        }
    };

    const {isOpen, onClose, alt, className, style, ...rest} = props;

    return (
        <Modal isOpen={isOpen} onOpenChange={(open) => !open && onClose(null)}>
            <ModalDialog>
                <>
                    <ModalHeader>Crop Image</ModalHeader>
                    <ModalBody>
                        <Cropper
                            ref={cropperRef}
                            src={image}
                            style={{height: 400, width: "100%"}}
                            initialAspectRatio={1}
                            aspectRatio={1}
                            minCropBoxHeight={128}
                            minCropBoxWidth={128}
                            responsive={true}
                            draggable={true}
                            enable={true}
                            dragMode={"move"}
                            checkOrientation={false} // https://github.com/fengyuanchen/cropperjs/issues/671
                            guides={true}
                            {...rest}
                        />
                    </ModalBody>
                    <ModalFooter>
                        <Button onPress={getCropData}>Save</Button>
                        <Button
                            onPress={() => props.onClose(null)}
                            variant="danger"
                        >
                            Cancel
                        </Button>
                    </ModalFooter>
                </>
            </ModalDialog>
        </Modal>
    );
}